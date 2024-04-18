use serde::{Serialize, Deserialize};

use fofo_utils::usizedb;


#[derive(Debug, Serialize, Deserialize)]
pub struct GetUserNotificationsQuery {
    #[serde(default)]
    pub index: usizedb,
    pub limit: usizedb,
    pub only_unread: bool,
    #[serde(default)]
    pub extended: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SetUserNotificationReadedQuery {
    pub id: usizedb,
    pub readed: bool,
}