# Étape 1 : Utiliser l'image Rust 1.80
FROM rust:1.80

# Définir le répertoire de travail
WORKDIR /usr/src/aquafarm-backend

# Copier le code source dans le conteneur
COPY . .

# Nettoyer, compiler et exécuter
CMD ["sh", "-c", "cargo clean && cargo build && cargo run"]
