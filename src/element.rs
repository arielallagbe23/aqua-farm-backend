use serde::{Deserialize, Serialize};
use sqlx::{mysql::MySqlPool, FromRow, Error};

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Element {
    pub id: i32,                   // ID unique de l'élément
    pub exploitation_id: i32,      // Référence à l'exploitation
    pub nom_element: String,       // Nom de l'élément
    pub quantite: i32,             // Quantité de l'élément
}

impl Element {
    // Ajouter un nouvel élément
    pub async fn create(
        pool: &MySqlPool,
        exploitation_id: i32,
        nom_element: String,
        quantite: i32,
    ) -> Result<Self, Error> {
        let insert_result = sqlx::query!(
            r#"
            INSERT INTO elements (exploitation_id, nom_element, quantite)
            VALUES (?, ?, ?)
            "#,
            exploitation_id,
            nom_element,
            quantite
        )
        .execute(pool)
        .await?;

        let last_id = insert_result.last_insert_id() as i32;

        Ok(Element {
            id: last_id,
            exploitation_id,
            nom_element,
            quantite,
        })
    }

    // Récupérer tous les éléments
    pub async fn get_all(pool: &MySqlPool) -> Result<Vec<Self>, Error> {
        let elements = sqlx::query_as!(
            Element,
            r#"
            SELECT id, exploitation_id, nom_element, quantite
            FROM elements
            "#
        )
        .fetch_all(pool)
        .await?;

        Ok(elements)
    }

    // Récupérer les éléments d'une exploitation spécifique
    pub async fn get_by_exploitation_id(
        pool: &MySqlPool,
        exploitation_id: i32,
    ) -> Result<Vec<Self>, Error> {
        let elements = sqlx::query_as!(
            Element,
            r#"
            SELECT id, exploitation_id, nom_element, quantite
            FROM elements
            WHERE exploitation_id = ?
            "#,
            exploitation_id
        )
        .fetch_all(pool)
        .await?;

        Ok(elements)
    }

    // Supprimer un élément
    pub async fn delete(pool: &MySqlPool, id: i32) -> Result<(), Error> {
        sqlx::query!(
            r#"
            DELETE FROM elements WHERE id = ?
            "#,
            id
        )
        .execute(pool)
        .await?;

        Ok(())
    }
}
