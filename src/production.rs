use chrono::NaiveDate;
use serde::{Serialize, Deserialize};
use sqlx::{mysql::MySqlPool, FromRow};

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Production {
    pub id: i32,                   // ID unique de la production
    pub element_id: i32,           // ID de l'élément lié
    pub quantite_produite: i32,    // Quantité produite
    pub unite_production: String,  // Unité de mesure de la production
    pub date_de_production: NaiveDate, // Date de la production
}

impl Production {
    /// Récupérer toutes les productions pour un élément donné
    pub async fn get_by_element_id(
        pool: &MySqlPool,
        element_id: i32,
    ) -> Result<Vec<Self>, sqlx::Error> {
        let productions: Vec<Production> = sqlx::query_as!(
            Production,
            r#"
            SELECT id, element_id, quantite_produite, unite_production, date_de_production
            FROM production
            WHERE element_id = ?
            "#,
            element_id
        )
        .fetch_all(pool)
        .await?;

        Ok(productions)
    }

    /// Créer une nouvelle entrée de production
    pub async fn create(
        pool: &MySqlPool,
        element_id: i32,
        quantite_produite: i32,
        unite_production: String,
        date_de_production: NaiveDate,
    ) -> Result<Self, sqlx::Error> {
        let insert_result = sqlx::query!(
            r#"
            INSERT INTO production (element_id, quantite_produite, unite_production, date_de_production)
            VALUES (?, ?, ?, ?)
            "#,
            element_id,
            quantite_produite,
            unite_production,
            date_de_production
        )
        .execute(pool)
        .await?;

        let last_id = insert_result.last_insert_id() as i32;

        Ok(Production {
            id: last_id,
            element_id,
            quantite_produite,
            unite_production,
            date_de_production,
        })
    }

    /// Supprimer une production par ID
    pub async fn delete(
        pool: &MySqlPool,
        id: i32,
    ) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            DELETE FROM production WHERE id = ?
            "#,
            id
        )
        .execute(pool)
        .await?;

        Ok(())
    }
}
