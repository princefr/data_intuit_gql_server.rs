use crate::enums::role::Role;
use crate::structs::user::User;
use tokio_postgres::Error;

pub trait UserTrait {
    /*
    * get user roles
    @param user_uid: &str
    @return Vec<Role>
    */
    async fn get_user_roles<'a>(&self, user_uid: &'a str) -> Result<Vec<Role>, Error>;
    /*
    * save user roles
    @param user_uid: &str
    @param roles: Vec<Role>
    */
    async fn save_user_role<'a>(&self, user_uid: &'a str, roles: &'a Role) -> Result<(), Error>;
    /*
    * create user
    @param user: User
    @return User

    */
    async fn create_user<'a>(&self, user: &'a User) -> Result<User, Error>;
    /*
    * update user name
    @param user_name: String
    @return User

    */
    async fn update_user_name<'a>(
        &self,
        user_name: &'a str,
        user_uid: &'a str,
    ) -> Result<User, Error>;

    /*
    * get user
    @param user_uid: &str
    @return User
    */
    async fn get_user<'a>(&self, user_uid: &'a str) -> Result<User, Error>;

    /*
     * crate random user into the database
     */
    #[cfg(test)]
    async fn crate_random_user<'a>(&self) -> Result<User, Error>;

    /*
     * crate test user for mutation and query testing
     */
    #[cfg(test)]
    async fn create_test_user<'a>(&self, uuid: &'a str) -> Result<User, Error>;
}
