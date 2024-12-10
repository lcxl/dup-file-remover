#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dup_file_remover::run().await
}
