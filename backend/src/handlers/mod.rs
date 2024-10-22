pub async fn handler() -> axum::response::Html<&'static str> {
    axum::response::Html("<h1>Hello, World!</h1>")
}
