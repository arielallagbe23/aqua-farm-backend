use actix_web::{web, App, HttpServer, Responder, HttpResponse, HttpRequest};
use actix_cors::Cors;
use sqlx::mysql::MySqlPool;
use serde::Deserialize;
use serde::Serialize;
use dotenv::dotenv;
use std::env;
use bcrypt::{hash, DEFAULT_COST};

mod type_user;
use type_user::TypeUser;

mod user;
use user::User;

mod domaine;
use domaine::Domaine;

mod type_exploitation;
use type_exploitation::TypeExploitation;

mod exploitation;
use exploitation::Exploitation;

mod type_element;
use type_element::TypeElement;

mod element;
use element::Element;

mod production;
use production::Production;

#[derive(Deserialize)]
struct CreateTypeUser {
    nom_type_user: String,
}

async fn hello_world() -> impl Responder {
    "Bienvenue sur AquaFarm API"
}

async fn add_type_user(
    pool: web::Data<MySqlPool>,
    form: web::Json<CreateTypeUser>,
) -> impl Responder {
    let type_user = TypeUser::create(pool.get_ref(), form.nom_type_user.clone()).await;

    match type_user {
        Ok(type_user) => HttpResponse::Ok().json(type_user),
        Err(e) => {
            println!("Erreur lors de l'ajout du type_user : {:?}", e);
            HttpResponse::InternalServerError().body("Erreur lors de l'ajout")
        },
    }
}

async fn get_all_type_user(pool: web::Data<MySqlPool>) -> impl Responder {
    match TypeUser::get_all(pool.get_ref()).await {
        Ok(types_user) => HttpResponse::Ok().json(types_user),
        Err(e) => {
            println!("Erreur lors de la récupération des types_user : {:?}", e);
            HttpResponse::InternalServerError().body("Erreur lors de la récupération des types")
        },
    }
}

#[derive(Deserialize)]
struct CreateUser {
    type_user_id: i32,
    nom: String,
    prenom: String,
    email: String,
    numero_telephone: String,
    mot_de_passe: String,
}

async fn add_user(
    pool: web::Data<MySqlPool>,
    form: web::Json<CreateUser>,
) -> impl Responder {
    let user = User::create(
        pool.get_ref(),
        form.type_user_id,
        form.nom.clone(),
        form.prenom.clone(),
        form.email.clone(),
        form.numero_telephone.clone(),
        form.mot_de_passe.clone(),
    )
    .await;

    match user {
        Ok(user) => HttpResponse::Ok().json(user),
        Err(e) => {
            println!("Erreur lors de l'ajout de l'utilisateur : {:?}", e);
            HttpResponse::InternalServerError().body("Erreur lors de l'ajout")
        },
    }
}

async fn get_users(pool: web::Data<MySqlPool>) -> impl Responder {
    match User::get_all(pool.get_ref()).await {
        Ok(users) => HttpResponse::Ok().json(users),
        Err(e) => {
            println!("Erreur lors de la récupération des utilisateurs : {:?}", e);
            HttpResponse::InternalServerError().body("Erreur lors de la récupération")
        },
    }
}

#[derive(Deserialize)]
struct LoginUser {
    email: String,
    mot_de_passe: String,
}

async fn login_user(
    pool: web::Data<MySqlPool>,
    form: web::Json<LoginUser>,
) -> impl Responder {
    match User::authenticate(pool.get_ref(), form.email.clone(), form.mot_de_passe.clone()).await {
        Ok((user, token)) => HttpResponse::Ok().json(serde_json::json!({
            "user": {
                "id": user.id,
                "nom": user.nom,
                "prenom": user.prenom,
                "email": user.email,
                "numero_telephone": user.numero_telephone,
                "type_user_id": user.type_user_id
            },
            "token": token
        })),
        Err(_) => HttpResponse::Unauthorized().body("Email ou mot de passe incorrect"),
    }
}

#[derive(Deserialize)]
struct UpdateUser {
    nom: Option<String>,
    prenom: Option<String>,
    email: Option<String>,
    numero_telephone: Option<String>,
    mot_de_passe: Option<String>,
}

async fn update_user(
    pool: web::Data<MySqlPool>,
    req: HttpRequest,
    id: web::Path<i32>,
    form: web::Json<UpdateUser>,
) -> impl Responder {
    let claims = match user::validate_token(&req) {
        Ok(claims) => claims,
        Err(e) => return e.into(),
    };

    let user_id = *id;

    let user_email = sqlx::query_scalar!(
        "SELECT email FROM users WHERE id = ?",
        user_id
    )
    .fetch_one(pool.get_ref())
    .await;

    if let Ok(user_email) = user_email {
        if claims.sub != user_email {
            return HttpResponse::Unauthorized().body("Non autorisé");
        }
    } else {
        return HttpResponse::Unauthorized().body("Utilisateur introuvable");
    }

    let mut query = String::from("UPDATE users SET ");
    let mut params = Vec::new();

    if let Some(nom) = &form.nom {
        query.push_str("nom = ?, ");
        params.push(nom.clone());
    }
    if let Some(prenom) = &form.prenom {
        query.push_str("prenom = ?, ");
        params.push(prenom.clone());
    }
    if let Some(email) = &form.email {
        query.push_str("email = ?, ");
        params.push(email.clone());
    }
    if let Some(numero_telephone) = &form.numero_telephone {
        query.push_str("numero_telephone = ?, ");
        params.push(numero_telephone.clone());
    }
    if let Some(mot_de_passe) = &form.mot_de_passe {
        let hashed_password = hash(mot_de_passe, DEFAULT_COST).unwrap();
        query.push_str("mot_de_passe = ?, ");
        params.push(hashed_password);
    }

    if query.ends_with(", ") {
        query.truncate(query.len() - 2);
    }

    query.push_str(" WHERE id = ?");
    params.push(user_id.to_string());

    let mut query_builder = sqlx::query(&query);
    for param in params {
        query_builder = query_builder.bind(param);
    }

    let result = query_builder.execute(pool.get_ref()).await;

    match result {
        Ok(_) => HttpResponse::Ok().body("Utilisateur mis à jour avec succès"),
        Err(err) => {
            println!("Erreur SQL: {:?}", err);
            HttpResponse::InternalServerError().body("Erreur lors de la mise à jour")
        }
    }
}

async fn delete_user(
    pool: web::Data<MySqlPool>,
    req: HttpRequest,
    id: web::Path<i32>,
) -> impl Responder {
    let claims = match user::validate_token(&req) {
        Ok(claims) => claims,
        Err(e) => return e.into(),
    };

    let user_id = *id;
    if claims.sub != user_id.to_string() {
        return HttpResponse::Unauthorized().body("Non autorisé");
    }

    let result = sqlx::query!("DELETE FROM users WHERE id = ?", user_id)
        .execute(pool.get_ref())
        .await;

    match result {
        Ok(_) => HttpResponse::Ok().body("Utilisateur supprimé avec succès"),
        Err(_) => HttpResponse::InternalServerError().body("Erreur lors de la suppression"),
    }
}

async fn get_user_by_id(
    pool: web::Data<MySqlPool>,
    id: web::Path<i32>,
) -> impl Responder {
    match User::get_by_id(pool.get_ref(), id.into_inner()).await {
        Ok(user) => HttpResponse::Ok().json(user),
        Err(_) => HttpResponse::NotFound().body("Utilisateur introuvable"),
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateDomaine {
    pub user_id: i32,
    pub nom_domaine: String,
}

// Ajouter un domaine
async fn add_domaine(
    pool: web::Data<MySqlPool>,
    form: web::Json<CreateDomaine>,
) -> impl Responder {
    match Domaine::create(pool.get_ref(), form.user_id, form.nom_domaine.clone()).await {
        Ok(domaine) => HttpResponse::Ok().json(domaine),
        Err(_) => HttpResponse::InternalServerError().body("Erreur lors de la création du domaine"),
    }
}


// Récupérer tous les domaines
async fn get_domaines(pool: web::Data<MySqlPool>) -> impl Responder {
    match Domaine::get_all(pool.get_ref()).await {
        Ok(domaines) => HttpResponse::Ok().json(domaines),
        Err(_) => HttpResponse::InternalServerError().body("Erreur lors de la récupération des domaines"),
    }
}

// Mettre à jour un domaine
async fn update_domaine(
    pool: web::Data<MySqlPool>,
    id: web::Path<i32>,
    form: web::Json<Domaine>,
) -> impl Responder {
    match Domaine::update_domaine(pool.get_ref(), *id, Some(form.nom_domaine.clone())).await {
        Ok(_) => HttpResponse::Ok().body("Domaine mis à jour avec succès"),
        Err(_) => HttpResponse::InternalServerError().body("Erreur lors de la mise à jour du domaine"),
    }
}

// Supprimer un domaine
async fn delete_domaine(
    pool: web::Data<MySqlPool>,
    id: web::Path<i32>,
) -> impl Responder {
    match Domaine::delete_domaine(pool.get_ref(), *id).await {
        Ok(_) => HttpResponse::Ok().body("Domaine supprimé avec succès"),
        Err(_) => HttpResponse::InternalServerError().body("Erreur lors de la suppression du domaine"),
    }
}

// Récupérer tous les domaines par user_id
async fn get_domaines_by_user_id(
    pool: web::Data<MySqlPool>,
    user_id: web::Path<i32>,
) -> impl Responder {
    match Domaine::get_all_by_user_id(pool.get_ref(), *user_id).await {
        Ok(domaines) => HttpResponse::Ok().json(domaines),
        Err(_) => HttpResponse::InternalServerError().body("Erreur lors de la récupération des domaines"),
    }
}


#[derive(Deserialize)]
struct CreateTypeExploitation {
    nom_type_exploitation: String,
}

async fn add_type_exploitation(
    pool: web::Data<MySqlPool>,
    form: web::Json<CreateTypeExploitation>,
) -> impl Responder {
    match TypeExploitation::create(pool.get_ref(), form.nom_type_exploitation.clone()).await {
        Ok(type_exploitation) => HttpResponse::Ok().json(type_exploitation),
        Err(_) => HttpResponse::InternalServerError().body("Erreur lors de l'ajout"),
    }
}

async fn get_all_types_exploitation(pool: web::Data<MySqlPool>) -> impl Responder {
    match TypeExploitation::get_all(pool.get_ref()).await {
        Ok(types_exploitation) => HttpResponse::Ok().json(types_exploitation),
        Err(_) => HttpResponse::InternalServerError().body("Erreur lors de la récupération"),
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateExploitationRequest {
    pub type_exploitation_id: i32,
    pub domaine_id: i32,
    pub nom_exploitation: String,
}

// Ajouter une exploitation
async fn add_exploitation(
    pool: web::Data<MySqlPool>,
    form: web::Json<CreateExploitationRequest>,
) -> impl Responder {
    match Exploitation::create(
        pool.get_ref(),
        form.type_exploitation_id,
        form.domaine_id,
        form.nom_exploitation.clone(),
    )
    .await
    {
        Ok(exploitation) => HttpResponse::Ok().json(exploitation),
        Err(_) => HttpResponse::InternalServerError().body("Erreur lors de la création de l'exploitation"),
    }
}

// Récupérer toutes les exploitations
async fn get_all_exploitations(pool: web::Data<MySqlPool>) -> impl Responder {
    match Exploitation::get_all(pool.get_ref()).await {
        Ok(exploitations) => HttpResponse::Ok().json(exploitations),
        Err(_) => HttpResponse::InternalServerError().body("Erreur lors de la récupération des exploitations"),
    }
}

// Supprimer une exploitation par ID
async fn delete_exploitation(
    pool: web::Data<MySqlPool>,
    id: web::Path<i32>,
) -> impl Responder {
    match Exploitation::delete(pool.get_ref(), *id).await {
        Ok(_) => HttpResponse::Ok().body("Exploitation supprimée avec succès"),
        Err(_) => HttpResponse::InternalServerError().body("Erreur lors de la suppression de l'exploitation"),
    }
}

// Récupérer toutes les exploitations d'un domaine
async fn get_exploitations_by_domaine(
    pool: web::Data<MySqlPool>,
    domaine_id: web::Path<i32>,
) -> impl Responder {
    match Exploitation::get_by_domaine_id(pool.get_ref(), *domaine_id).await {
        Ok(exploitations) => HttpResponse::Ok().json(exploitations),
        Err(_) => HttpResponse::InternalServerError().body("Erreur lors de la récupération des exploitations"),
    }
}

#[derive(Deserialize)]
struct CreateTypeElement {
    nom_type_element: String,
}

// Ajouter un type d'élément
async fn add_type_element(
    pool: web::Data<MySqlPool>,
    form: web::Json<CreateTypeElement>,
) -> impl Responder {
    match TypeElement::create(pool.get_ref(), form.nom_type_element.clone()).await {
        Ok(type_element) => HttpResponse::Ok().json(type_element),
        Err(_) => HttpResponse::InternalServerError().body("Erreur lors de la création du type d'élément"),
    }
}

// Récupérer tous les types d'éléments
async fn get_all_type_elements(pool: web::Data<MySqlPool>) -> impl Responder {
    match TypeElement::get_all(pool.get_ref()).await {
        Ok(type_elements) => HttpResponse::Ok().json(type_elements),
        Err(_) => HttpResponse::InternalServerError().body("Erreur lors de la récupération des types d'éléments"),
    }
}

// Supprimer un type d'élément
async fn delete_type_element(
    pool: web::Data<MySqlPool>,
    id: web::Path<i32>,
) -> impl Responder {
    match TypeElement::delete(pool.get_ref(), *id).await {
        Ok(_) => HttpResponse::Ok().body("Type d'élément supprimé avec succès"),
        Err(_) => HttpResponse::InternalServerError().body("Erreur lors de la suppression"),
    }
}

// Mettre à jour un type d'élément
async fn update_type_element(
    pool: web::Data<MySqlPool>,
    id: web::Path<i32>,
    form: web::Json<CreateTypeElement>,
) -> impl Responder {
    match TypeElement::update(pool.get_ref(), *id, form.nom_type_element.clone()).await {
        Ok(_) => HttpResponse::Ok().body("Type d'élément mis à jour avec succès"),
        Err(_) => HttpResponse::InternalServerError().body("Erreur lors de la mise à jour"),
    }
}

#[derive(Deserialize)]
struct CreateElement {
    exploitation_id: i32,
    nom_element: String,
    quantite: i32,
}

// Ajouter un nouvel élément
async fn add_element(
    pool: web::Data<MySqlPool>,
    form: web::Json<CreateElement>,
) -> impl Responder {
    match Element::create(
        pool.get_ref(),
        form.exploitation_id,
        form.nom_element.clone(),
        form.quantite,
    )
    .await
    {
        Ok(element) => HttpResponse::Ok().json(element),
        Err(_) => HttpResponse::InternalServerError().body("Erreur lors de la création de l'élément"),
    }
}

// Récupérer tous les éléments
async fn get_all_elements(pool: web::Data<MySqlPool>) -> impl Responder {
    match Element::get_all(pool.get_ref()).await {
        Ok(elements) => HttpResponse::Ok().json(elements),
        Err(_) => HttpResponse::InternalServerError().body("Erreur lors de la récupération des éléments"),
    }
}

// Récupérer les éléments d'une exploitation spécifique
async fn get_elements_by_exploitation(
    pool: web::Data<MySqlPool>,
    exploitation_id: web::Path<i32>,
) -> impl Responder {
    match Element::get_by_exploitation_id(pool.get_ref(), *exploitation_id).await {
        Ok(elements) => HttpResponse::Ok().json(elements),
        Err(_) => HttpResponse::InternalServerError().body("Erreur lors de la récupération des éléments"),
    }
}

// Supprimer un élément
async fn delete_element(
    pool: web::Data<MySqlPool>,
    id: web::Path<i32>,
) -> impl Responder {
    match Element::delete(pool.get_ref(), *id).await {
        Ok(_) => HttpResponse::Ok().body("Élément supprimé avec succès"),
        Err(_) => HttpResponse::InternalServerError().body("Erreur lors de la suppression de l'élément"),
    }
}

async fn get_productions_by_element_id(
    pool: web::Data<MySqlPool>,
    element_id: web::Path<i32>,
) -> impl Responder {
    match Production::get_by_element_id(pool.get_ref(), *element_id).await {
        Ok(productions) => HttpResponse::Ok().json(productions),
        Err(_) => HttpResponse::InternalServerError().body("Erreur lors de la récupération des productions"),
    }
}

async fn get_domaines_for_user(
    pool: web::Data<MySqlPool>,
    req: HttpRequest,
) -> impl Responder {
    // Récupérer les claims du token
    let claims = match user::validate_token(&req) {
        Ok(claims) => claims,
        Err(e) => return e.into(), // Retourne une erreur 401 si le token est invalide
    };

    // Récupérer l'utilisateur connecté à partir des claims
    let user_email = claims.sub;

    // Optionnel : Trouver l'user_id à partir de l'email (si user_id n'est pas dans le JWT)
    let user_id_result = sqlx::query_scalar!(
        r#"
        SELECT id FROM users WHERE email = ?
        "#,
        user_email
    )
    .fetch_one(pool.get_ref())
    .await;

    let user_id = match user_id_result {
        Ok(id) => id,
        Err(_) => return HttpResponse::InternalServerError().body("Erreur : utilisateur non trouvé"),
    };

    // Récupérer les domaines pour cet utilisateur
    let domaines = Domaine::get_all_by_user_id(pool.get_ref(), user_id).await;

    match domaines {
        Ok(domaines) => HttpResponse::Ok().json(domaines),
        Err(_) => HttpResponse::InternalServerError().body("Erreur lors de la récupération des domaines"),
    }
}

async fn get_connected_user(
    pool: web::Data<MySqlPool>,
    req: HttpRequest,
) -> impl Responder {
    // Valider le token JWT pour récupérer les claims
    let claims = match user::validate_token(&req) {
        Ok(claims) => claims,
        Err(e) => return e.into(), // Retourne une erreur 401 si le token est invalide
    };

    // Extraire l'email (ou autre identifiant) des claims
    let user_email = claims.sub;

    // Récupérer les informations de l'utilisateur à partir de l'email
    let user = sqlx::query_as!(
        User,
        r#"
        SELECT id, type_user_id, nom, prenom, email, numero_telephone, mot_de_passe
        FROM users
        WHERE email = ?
        "#,
        user_email
    )
    .fetch_one(pool.get_ref())
    .await;

    match user {
        Ok(user) => HttpResponse::Ok().json(user), // Retourne les informations de l'utilisateur
        Err(_) => HttpResponse::NotFound().body("Utilisateur non trouvé"), // Retourne une erreur si l'utilisateur n'est pas trouvé
    }
}

async fn add_domaine_for_user(
    pool: web::Data<MySqlPool>,
    req: HttpRequest,
    form: web::Json<CreateDomaine>,
) -> impl Responder {
    // Récupérer les claims à partir du token JWT
    let claims = match user::validate_token(&req) {
        Ok(claims) => claims,
        Err(e) => return e.into(), // Retourne une erreur 401 si le token est invalide
    };

    // Extraire l'email de l'utilisateur connecté depuis les claims
    let user_email = claims.sub;

    // Trouver l'ID de l'utilisateur à partir de l'email
    let user_id_result = sqlx::query_scalar!(
        "SELECT id FROM users WHERE email = ?",
        user_email
    )
    .fetch_one(pool.get_ref())
    .await;

    let user_id = match user_id_result {
        Ok(id) => id,
        Err(_) => return HttpResponse::InternalServerError().body("Erreur : utilisateur non trouvé"),
    };

    // Utiliser la méthode `create` pour insérer le domaine
    match Domaine::create(pool.get_ref(), user_id, form.nom_domaine.clone()).await {
        Ok(domaine) => HttpResponse::Ok().json(domaine), // Retourne le domaine ajouté
        Err(_) => HttpResponse::InternalServerError().body("Erreur lors de la création du domaine"),
    }
}




#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL doit être défini");

    let pool = MySqlPool::connect(&database_url)
        .await
        .expect("Impossible de se connecter à la base de données");

    println!("Connexion réussie à la base de données.");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            // Ajout du middleware CORS
            .wrap(
                Cors::default()
                    .allow_any_origin() // Autorise toutes les origines
                    .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"]) // Méthodes autorisées
                    .allowed_headers(vec![
                        actix_web::http::header::CONTENT_TYPE,
                        actix_web::http::header::AUTHORIZATION,
                    ]) // Autorise Content-Type et Authorization
                    .max_age(3600), // Cache des options CORS pendant 1 heure
            )
            // Routes existantes
            .route("/", web::get().to(hello_world))
            .route("/type_user", web::get().to(get_all_type_user))
            .route("/type_user", web::post().to(add_type_user))
            .route("/users", web::post().to(add_user))
            .route("/users", web::get().to(get_users))
            .route("/login", web::post().to(login_user))

            .route("/users/{id}", web::put().to(update_user))
            .route("/users/{id}", web::delete().to(delete_user))
            .route("/users/{id}", web::get().to(get_user_by_id))
            .route("/users/user/connected", web::get().to(get_connected_user))
            

            .route("/domaines", web::post().to(add_domaine))
            .route("/domaines/user/{user_id}", web::get().to(get_domaines_by_user_id))
            .route("/domaines", web::get().to(get_domaines))
            .route("/domaines/{id}", web::put().to(update_domaine))
            .route("/domaines/{id}", web::delete().to(delete_domaine))
            .route("/domaines/user", web::get().to(get_domaines_for_user))
            .route("/domaines/user/add", web::post().to(add_domaine_for_user))



            .route("/type_exploitation", web::post().to(add_type_exploitation))
            .route("/type_exploitation", web::get().to(get_all_types_exploitation))

            .route("/exploitations", web::post().to(add_exploitation))
            .route("/exploitations", web::get().to(get_all_exploitations))
            .route("/exploitations/{id}", web::delete().to(delete_exploitation))
            .route("/exploitations/domaine/{domaine_id}", web::get().to(get_exploitations_by_domaine))

            .route("/type_elements", web::post().to(add_type_element))
            .route("/type_elements", web::get().to(get_all_type_elements))
            .route("/type_elements/{id}", web::delete().to(delete_type_element))
            .route("/type_elements/{id}", web::put().to(update_type_element))

            .route("/elements", web::post().to(add_element))
            .route("/elements", web::get().to(get_all_elements))
            .route("/elements/exploitation/{exploitation_id}", web::get().to(get_elements_by_exploitation))
            .route("/elements/{id}", web::delete().to(delete_element))

            .route("/productions/element/{element_id}", web::get().to(get_productions_by_element_id))

    })
    .bind("127.0.0.1:5005")?
    .run()
    .await
}