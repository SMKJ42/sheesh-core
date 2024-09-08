use std::error;

use crate::{
    harness::{
        sqlite::{SqliteDiskOpSession, SqliteDiskOpToken},
        DiskOp, IntoRow,
    },
    hash::{DefaultHashGenerator, HashGenerator},
};

use super::{
    auth_token::{AuthToken, AuthTokenManager, AuthTokenManagerConfig},
    id::{DefaultIdGenerator, IdGenerator},
};

pub struct SessionManagerConfig<T, U>
where
    T: IdGenerator,
    U: HashGenerator,
{
    id_generator: T,
    token_generator_config: AuthTokenManagerConfig<T, U>,
    fields: Vec<String>,
}

impl SessionManagerConfig<DefaultIdGenerator, DefaultHashGenerator> {
    pub fn new_default() -> Self {
        return Self {
            id_generator: DefaultIdGenerator {},
            token_generator_config: AuthTokenManagerConfig::new_default(),
            fields: vec![
                "id".to_string(),
                "user_id".to_string(),
                "expires".to_string(),
                "refresh_token".to_string(),
            ],
        };
    }

    pub fn with_fields(&mut self, fields: Vec<String>) {
        self.fields.extend(fields);
    }
}

impl<T, U> SessionManagerConfig<T, U>
where
    T: IdGenerator + Copy,
    U: HashGenerator + Copy,
{
    pub fn init<V: DiskOp, X: DiskOp>(
        &self,
        session_harness: V,
        token_harness: X,
    ) -> SessionManager<T, U, V, X> {
        return SessionManager {
            id_generator: self.id_generator,
            token_generator: self.token_generator_config.init(token_harness),
            harness: session_harness,
            fields: self.fields.to_owned(),
        };
    }
}

pub struct SessionManager<T, U, V, X>
where
    T: IdGenerator,
    U: HashGenerator,
    V: DiskOp,
    X: DiskOp,
{
    id_generator: T,
    token_generator: AuthTokenManager<T, U, X>,
    fields: Vec<String>,
    harness: V,
}

impl
    SessionManager<DefaultIdGenerator, DefaultHashGenerator, SqliteDiskOpSession, SqliteDiskOpToken>
{
}

impl<T, U, X> SessionManager<T, U, SqliteDiskOpSession, X>
where
    T: IdGenerator,
    U: HashGenerator,
    X: DiskOp,
{
}

impl<T, U, V, X> SessionManager<T, U, V, X>
where
    T: IdGenerator,
    U: HashGenerator,
    V: DiskOp,
    X: DiskOp,
{
    pub fn new_session(&self, user_id: u64) -> Result<(Session, AuthToken), Box<dyn error::Error>> {
        let id = self.id_generator.new_u64();
        let new_token = self.token_generator.next_token(id)?;

        let session = Session {
            id,
            user_id,
            current_token_id: new_token.id(),
        };

        self.harness.insert(&session, &self.fields)?;

        return Ok((session, new_token));
    }

    pub fn refresh_session_token(
        &self,
        session: &mut Session,
    ) -> Result<AuthToken, Box<dyn error::Error>> {
        let new_token = self.token_generator.next_token(session.id)?;
        session.current_token_id = new_token.id();

        return Ok(new_token);
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Session {
    id: u64,
    user_id: u64,
    current_token_id: u64,
}

impl IntoRow for Session {
    fn into_row(&self) -> Vec<String> {
        return vec![
            self.id.to_string(),
            self.user_id.to_string(),
            self.current_token_id.to_string(),
        ];
    }
}

/// Session
///
/// Only has getters to prevent accidental overwrites.
impl Session {
    pub fn id(&self) -> u64 {
        return self.id;
    }

    pub fn user_id(&self) -> u64 {
        return self.user_id;
    }

    pub fn current_token_id(&self) -> u64 {
        return self.current_token_id;
    }
}
