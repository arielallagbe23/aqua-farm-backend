use serde::{Deserialize, Serialize};
use sqlx::{mysql::MySqlPool, FromRow, Error};

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct TypeUser {
    pub id: i32,                 // Non nullable
    pub nom_type_user: String,   // Nom du type utilisateur
}


impl TypeUser {
    // Ajouter un type d'utilisateur
    pub async fn create(pool: &MySqlPool, nom_type_user: String) -> Result<Self, Error> {
        let insert_result = sqlx::query!(
            r#"
            INSERT INTO types_user (nom_type_user)
            VALUES (?)
            "#,
            nom_type_user
        )
        .execute(pool)
        .await?;

        // Récupérer l'ID de l'utilisateur inséré
        let last_id = insert_result.last_insert_id() as i32;

        Ok(TypeUser {
            id: last_id,
            nom_type_user,
        })
    }

    // Récupérer tous les types d'utilisateur
    pub async fn get_all(pool: &MySqlPool) -> Result<Vec<Self>, Error> {
        let types_user = sqlx::query_as!(
            TypeUser,
            r#"
            SELECT id, nom_type_user
            FROM types_user
            "#
        )
        .fetch_all(pool)
        .await?;

        Ok(types_user)
    }

}
