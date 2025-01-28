use actix_web::{Error as ActixError, HttpRequest};
use sqlx::{mysql::MySqlPool, FromRow, Error as SqlxError};
use serde::{Deserialize, Serialize};
use chrono::Utc;
use bcrypt::{hash, verify, DEFAULT_COST};
use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey};



#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: i32,                    // ID unique
    pub type_user_id: i32,          // Référence au type d'utilisateur
    pub nom: String,                // Nom de l'utilisateur
    pub prenom: String,             // Prénom de l'utilisateur
    pub email: String,              // Adresse email unique
    pub numero_telephone: String,   // Numéro de téléphone
    pub mot_de_passe: String,       // Mot de passe (haché)
}

// Structure pour le contenu du JWT
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // Identifiant principal (par ex. email ou user ID)
    pub exp: usize,  // Expiration du token (en timestamp Unix)
}

pub fn validate_token(req: &HttpRequest) -> Result<Claims, ActixError> {
    let jwt_secret = std::env::var("JWT_SECRET").expect("JWT_SECRET doit être défini");

    let token = req
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
        .ok_or(actix_web::error::ErrorUnauthorized("Token manquant"))?;

    println!("Received token: {}", token); // Debug

    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(jwt_secret.as_ref()),
        &Validation::default(),
    )
    .map_err(|e| {
        println!("Token validation error: {:?}", e); // Debug
        actix_web::error::ErrorUnauthorized("Token invalide")
    })?;

    Ok(token_data.claims)
}

impl User {
    /// Ajouter un utilisateur avec hachage du mot de passe
    pub async fn create(
        pool: &MySqlPool,
        type_user_id: i32,
        nom: String,
        prenom: String,
        email: String,
        numero_telephone: String,
        mot_de_passe: String,
    ) -> Result<Self, SqlxError> { // Utilisation de SqlxError
        let hashed_password = hash(mot_de_passe, DEFAULT_COST)
            .map_err(|_| SqlxError::RowNotFound)?;
    
        let insert_result = sqlx::query!(
            r#"
            INSERT INTO users (type_user_id, nom, prenom, email, numero_telephone, mot_de_passe)
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
            type_user_id,
            nom,
            prenom,
            email,
            numero_telephone,
            hashed_password
        )
        .execute(pool)
        .await?;
    
        let last_id = insert_result.last_insert_id() as i32;
    
        Ok(User {
            id: last_id,
            type_user_id,
            nom,
            prenom,
            email,
            numero_telephone,
            mot_de_passe: hashed_password,
        })
    }
    

    /// Vérifier les identifiants de l'utilisateur (email + mot de passe)
    pub async fn authenticate(
        pool: &MySqlPool,
        email: String,
        mot_de_passe: String,
    ) -> Result<(Self, String), SqlxError> { // Utilisation de SqlxError
        let user = sqlx::query_as!(
            User,
            r#"
            SELECT id, type_user_id, nom, prenom, email, numero_telephone, mot_de_passe
            FROM users
            WHERE email = ?
            "#,
            email
        )
        .fetch_one(pool)
        .await?;
    
        let is_valid = verify(mot_de_passe, &user.mot_de_passe).map_err(|_| SqlxError::RowNotFound)?;
    
        if is_valid {
            let jwt_secret = std::env::var("JWT_SECRET").expect("JWT_SECRET doit être défini");
            
            // Définir l'expiration à 48 heures
            let expiration = Utc::now()
                .checked_add_signed(chrono::Duration::days(2)) // 48 heures
                .expect("Erreur de génération de la durée")
                .timestamp() as usize;
    
            let claims = Claims {
                sub: user.email.clone(),
                exp: expiration,
            };
    
            let token = encode(
                &Header::default(),
                &claims,
                &EncodingKey::from_secret(jwt_secret.as_ref()),
            )
            .map_err(|_| SqlxError::RowNotFound)?;
    
            Ok((user, token))
        } else {
            Err(SqlxError::RowNotFound)
        }
    }
    
    

    /// Récupérer tous les utilisateurs
    pub async fn get_all(pool: &MySqlPool) -> Result<Vec<Self>, SqlxError> {
        let users = sqlx::query_as!(
            User,
            r#"
            SELECT id, type_user_id, nom, prenom, email, numero_telephone, mot_de_passe
            FROM users
            "#
        )
        .fetch_all(pool)
        .await?;
    
        Ok(users)
    }

    /// Récupérer un utilisateur par son ID
    pub async fn get_by_id(pool: &MySqlPool, user_id: i32) -> Result<Self, SqlxError> {
        let user = sqlx::query_as!(
            User,
            r#"
            SELECT id, type_user_id, nom, prenom, email, numero_telephone, mot_de_passe
            FROM users
            WHERE id = ?
            "#,
            user_id
        )
        .fetch_one(pool)
        .await?;
    
        Ok(user)
    }

}
