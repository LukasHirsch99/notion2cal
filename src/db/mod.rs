use sqlx::PgPool;

pub mod calendar;
pub mod session;
pub mod user;

pub struct Repo {
    db: PgPool,
}

impl Repo {
    pub fn new(db: PgPool) -> Self {
        Repo { db }
    }
}
