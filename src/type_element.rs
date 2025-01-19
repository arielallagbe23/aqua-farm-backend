use serde::{Deserialize, Serialize};
use sqlx::{mysql::MySqlPool, FromRow, Error};

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct TypeElement {
    pub id: i32,                 // ID unique
    pub nom_type_element: String,     // Nom du type d'élément
}

impl TypeElement {
    // Créer un nouveau type d'élément
    pub async fn create(pool: &MySqlPool, nom_type_element: String) -> Result<Self, Error> {
        let insert_result = sqlx::query!(
            r#"
            INSERT INTO types_element (nom_type_element)
            VALUES (?)
            "#,
            nom_type_element
        )
        .execute(pool)
        .await?;

        let last_id = insert_result.last_insert_id() as i32;

        Ok(TypeElement {
            id: last_id,
            nom_type_element,
        })
    }

    // Récupérer tous les types d'éléments
    pub async fn get_all(pool: &MySqlPool) -> Result<Vec<Self>, Error> {
        let type_elements = sqlx::query_as!(
            TypeElement,
            r#"
            SELECT id, nom_type_element
            FROM types_element
            "#
        )
        .fetch_all(pool)
        .await?;

        Ok(type_elements)
    }

    // Supprimer un type d'élément
    pub async fn delete(pool: &MySqlPool, id: i32) -> Result<(), Error> {
        sqlx::query!(
            r#"
            DELETE FROM types_element WHERE id = ?
            "#,
            id
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    // Mettre à jour un type d'élément
    pub async fn update(pool: &MySqlPool, id: i32, nom_element: String) -> Result<(), Error> {
        sqlx::query!(
            r#"
            UPDATE types_element
            SET nom_type_element = ?
            WHERE id = ?
            "#,
            nom_element,
            id
        )
        .execute(pool)
        .await?;

        Ok(())
    }
}
