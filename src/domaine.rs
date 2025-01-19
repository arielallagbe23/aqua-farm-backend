use sqlx::{mysql::MySqlPool, FromRow, Error as SqlxError};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Domaine {
    pub id: i32,
    pub user_id: i32,
    pub nom_domaine: String,
}

impl Domaine {
    // Ajouter un domaine
    pub async fn create(
        pool: &MySqlPool,
        user_id: i32,
        nom_domaine: String,
    ) -> Result<Self, SqlxError> {
        // Insérer le domaine dans la base de données
        let insert_result = sqlx::query!(
            r#"
            INSERT INTO domaines (user_id, nom_domaine)
            VALUES (?, ?)
            "#,
            user_id,
            nom_domaine
        )
        .execute(pool)
        .await?;

        // Obtenir l'ID de la dernière insertion
        let last_id = insert_result.last_insert_id() as i32;

        // Retourner l'objet Domaine avec les informations
        Ok(Domaine {
            id: last_id,
            user_id,
            nom_domaine,
        })
    }

    // Récupérer tous les domaines
    pub async fn get_all(pool: &MySqlPool) -> Result<Vec<Self>, sqlx::Error> {
        let domaines = sqlx::query_as!(
            Domaine,
            r#"
            SELECT id, user_id, nom_domaine
            FROM domaines
            "#
        )
        .fetch_all(pool)
        .await?;
    
        Ok(domaines)
    }

    // Mettre à jour un domaine
    pub async fn update_domaine(
        pool: &MySqlPool,
        id: i32,
        nom_domaine: Option<String>,
    ) -> Result<(), sqlx::Error> {
        if let Some(nom) = nom_domaine {
            sqlx::query!(
                r#"
                UPDATE domaines
                SET nom_domaine = ?
                WHERE id = ?
                "#,
                nom,
                id
            )
            .execute(pool)
            .await?;
        }

        Ok(())
    }

    // Supprimer un domaine
    pub async fn delete_domaine(pool: &MySqlPool, id: i32) -> Result<(), sqlx::Error> {
        sqlx::query!(
            r#"
            DELETE FROM domaines WHERE id = ?
            "#,
            id
        )
        .execute(pool)
        .await?;
    
        Ok(())
    }

    // Récupérer tous les domaines pour un utilisateur donné
    pub async fn get_all_by_user_id(pool: &MySqlPool, user_id: i32) -> Result<Vec<Self>, sqlx::Error> {
        let domaines = sqlx::query_as!(
            Domaine,
            r#"
            SELECT id, user_id, nom_domaine
            FROM domaines
            WHERE user_id = ?
            "#,
            user_id
        )
        .fetch_all(pool)
        .await?;
    
        Ok(domaines)
    }
}
