use std::error;

use sheesh::{
    harness::{
        sqlite::{SqliteDiskOpSession, SqliteDiskOpToken, SqliteDiskOpUser},
        DiskOp, DiskOpManager,
    },
    session::SessionManagerConfig,
    user::{Group, PrivateUserMeta, PublicUserMeta, Role, User, UserManagerConfig},
};

extern crate r2d2;
extern crate r2d2_sqlite;
extern crate rusqlite;

use r2d2_sqlite::SqliteConnectionManager;

enum Roles {
    Admin,
}

impl Role for Roles {}

struct MyPublicUserMetadata {}
impl PublicUserMeta for MyPublicUserMetadata {}

struct MyPrivateUserMetadata {}
impl PrivateUserMeta for MyPrivateUserMetadata {}

struct SomeGroup {}
impl Group for SomeGroup {}

type MyUser = User<Roles, SomeGroup, MyPublicUserMetadata, MyPrivateUserMetadata>;

fn main() {
    // establish db connection.
    let conn_manager = SqliteConnectionManager::file("example_db/sqlite.db");
    let pool = r2d2::Pool::new(conn_manager).unwrap();

    // initalize db harness. if you would like to see how to implement your own, look inside the harness modules.
    let disk_manager = DiskOpManager::new_sqlite(pool);
    disk_manager.init_tables().unwrap();

    // provide db harness to function interfaces.
    let user_manager = UserManagerConfig::new_default().init(disk_manager.user);
    let session_manager =
        SessionManagerConfig::new_default().init(disk_manager.session, disk_manager.token);

    let mut i = 0;

    loop {
        let mut user: MyUser = user_manager
            .new_user(
                "test".to_string(),
                Roles::Admin,
                MyPublicUserMetadata {},
                MyPrivateUserMetadata {},
            )
            .unwrap();

        let public = MyPublicUserMetadata {};
        let private = MyPrivateUserMetadata {};

        user.set_public_data(public);
        user.set_private_data(private);

        let (mut session, mut token) = session_manager.new_session(user.public().id()).unwrap();

        token.invalidate();

        let mut new_token = session_manager.refresh_session_token(&mut session).unwrap();

        let is_valid_token = new_token.is_valid();
        if i % 100 == 0 {
            println!("one loop");
        }
        i += 1;
    }
}
