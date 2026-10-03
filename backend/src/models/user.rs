use serde::{Deserialize, Serialize};

#[allow(dead_code)] // Mencegah compiler menampilkan warning jika struct belum dipakai
#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub name: String,
    pub email: String,
}