# Utiliser une image officielle Rust pour construire l'application
FROM rust:latest as builder

# Définir le répertoire de travail
WORKDIR /app

# Copier les fichiers Cargo.toml et Cargo.lock en premier (pour la mise en cache)
COPY Cargo.toml Cargo.lock ./

# Créer uniquement les fichiers de dépendances pour les mettre en cache
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release

# Copier le reste du code source
COPY . .

# Compiler l'application en mode release
RUN cargo build --release

# Utiliser une image plus légère pour l'exécution
FROM debian:bullseye-slim

# Installer les dépendances nécessaires
RUN apt-get update && apt-get install -y libssl-dev ca-certificates && rm -rf /var/lib/apt/lists/*

# Créer un utilisateur non-root pour la sécurité
RUN useradd -m rustuser
USER rustuser

# Définir le répertoire de travail
WORKDIR /app

# Copier le binaire compilé depuis l'étape précédente
COPY --from=builder /app/target/release/my-backend .

# Exposer le port utilisé par l'application
EXPOSE 5005

# Démarrer l'application
CMD ["./my-backend"]
