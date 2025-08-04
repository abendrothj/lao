use actix_web::{web, App, HttpServer, HttpResponse, Result, middleware};
use actix_cors::Cors;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use std::fs;
use std::path::Path;
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize, Clone)]
struct PluginMetadata {
    id: String,
    name: String,
    version: String,
    description: String,
    author: String,
    license: String,
    repository: String,
    tags: Vec<String>,
    capabilities: Vec<PluginCapability>,
    dependencies: Vec<PluginDependency>,
    downloads: u64,
    rating: f64,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    download_url: Option<String>,
    documentation_url: Option<String>,
    compatible_versions: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct PluginCapability {
    name: String,
    description: String,
    input_type: String,
    output_type: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct PluginDependency {
    name: String,
    version: String,
    optional: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct SearchQuery {
    q: Option<String>,
    tags: Option<Vec<String>>,
    capabilities: Option<Vec<String>>,
    author: Option<String>,
    limit: Option<usize>,
    offset: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize)]
struct PluginUpload {
    name: String,
    version: String,
    description: String,
    author: String,
    license: String,
    repository: String,
    tags: Vec<String>,
    capabilities: Vec<PluginCapability>,
    dependencies: Vec<PluginDependency>,
    compatible_versions: Vec<String>,
}

struct AppState {
    plugins: Mutex<HashMap<String, PluginMetadata>>,
}

async fn list_plugins(
    state: web::Data<AppState>,
    query: web::Query<SearchQuery>,
) -> Result<HttpResponse> {
    let plugins = state.plugins.lock().unwrap();
    
    let mut filtered_plugins: Vec<&PluginMetadata> = plugins.values().collect();
    
    // Apply filters
    if let Some(search_q) = &query.q {
        filtered_plugins.retain(|p| {
            p.name.to_lowercase().contains(&search_q.to_lowercase()) ||
            p.description.to_lowercase().contains(&search_q.to_lowercase()) ||
            p.tags.iter().any(|t| t.to_lowercase().contains(&search_q.to_lowercase()))
        });
    }
    
    if let Some(tags) = &query.tags {
        filtered_plugins.retain(|p| {
            tags.iter().all(|tag| p.tags.contains(tag))
        });
    }
    
    if let Some(capabilities) = &query.capabilities {
        filtered_plugins.retain(|p| {
            capabilities.iter().all(|cap| {
                p.capabilities.iter().any(|c| c.name == *cap)
            })
        });
    }
    
    if let Some(author) = &query.author {
        filtered_plugins.retain(|p| {
            p.author.to_lowercase().contains(&author.to_lowercase())
        });
    }
    
    // Apply pagination
    let offset = query.offset.unwrap_or(0);
    let limit = query.limit.unwrap_or(50);
    let end = std::cmp::min(offset + limit, filtered_plugins.len());
    
    let paginated_plugins: Vec<&PluginMetadata> = filtered_plugins[offset..end].to_vec();
    
    Ok(HttpResponse::Ok().json(paginated_plugins))
}

async fn get_plugin(
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> Result<HttpResponse> {
    let plugin_id = path.into_inner();
    let plugins = state.plugins.lock().unwrap();
    
    if let Some(plugin) = plugins.get(&plugin_id) {
        Ok(HttpResponse::Ok().json(plugin))
    } else {
        Ok(HttpResponse::NotFound().json(serde_json::json!({
            "error": "Plugin not found"
        })))
    }
}

async fn upload_plugin(
    state: web::Data<AppState>,
    plugin_data: web::Json<PluginUpload>,
) -> Result<HttpResponse> {
    let mut plugins = state.plugins.lock().unwrap();
    
    let plugin_id = Uuid::new_v4().to_string();
    let now = Utc::now();
    
    let plugin = PluginMetadata {
        id: plugin_id.clone(),
        name: plugin_data.name.clone(),
        version: plugin_data.version.clone(),
        description: plugin_data.description.clone(),
        author: plugin_data.author.clone(),
        license: plugin_data.license.clone(),
        repository: plugin_data.repository.clone(),
        tags: plugin_data.tags.clone(),
        capabilities: plugin_data.capabilities.clone(),
        dependencies: plugin_data.dependencies.clone(),
        downloads: 0,
        rating: 0.0,
        created_at: now,
        updated_at: now,
        download_url: None,
        documentation_url: None,
        compatible_versions: plugin_data.compatible_versions.clone(),
    };
    
    plugins.insert(plugin_id.clone(), plugin);
    
    Ok(HttpResponse::Created().json(serde_json::json!({
        "id": plugin_id,
        "message": "Plugin uploaded successfully"
    })))
}

async fn update_plugin(
    state: web::Data<AppState>,
    path: web::Path<String>,
    plugin_data: web::Json<PluginUpload>,
) -> Result<HttpResponse> {
    let plugin_id = path.into_inner();
    let mut plugins = state.plugins.lock().unwrap();
    
    if let Some(plugin) = plugins.get_mut(&plugin_id) {
        plugin.name = plugin_data.name.clone();
        plugin.version = plugin_data.version.clone();
        plugin.description = plugin_data.description.clone();
        plugin.author = plugin_data.author.clone();
        plugin.license = plugin_data.license.clone();
        plugin.repository = plugin_data.repository.clone();
        plugin.tags = plugin_data.tags.clone();
        plugin.capabilities = plugin_data.capabilities.clone();
        plugin.dependencies = plugin_data.dependencies.clone();
        plugin.compatible_versions = plugin_data.compatible_versions.clone();
        plugin.updated_at = Utc::now();
        
        Ok(HttpResponse::Ok().json(serde_json::json!({
            "message": "Plugin updated successfully"
        })))
    } else {
        Ok(HttpResponse::NotFound().json(serde_json::json!({
            "error": "Plugin not found"
        })))
    }
}

async fn delete_plugin(
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> Result<HttpResponse> {
    let plugin_id = path.into_inner();
    let mut plugins = state.plugins.lock().unwrap();
    
    if plugins.remove(&plugin_id).is_some() {
        Ok(HttpResponse::Ok().json(serde_json::json!({
            "message": "Plugin deleted successfully"
        })))
    } else {
        Ok(HttpResponse::NotFound().json(serde_json::json!({
            "error": "Plugin not found"
        })))
    }
}

async fn download_plugin(
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> Result<HttpResponse> {
    let plugin_id = path.into_inner();
    let mut plugins = state.plugins.lock().unwrap();
    
    if let Some(plugin) = plugins.get_mut(&plugin_id) {
        plugin.downloads += 1;
        
        // In a real implementation, this would serve the actual plugin file
        Ok(HttpResponse::Ok().json(serde_json::json!({
            "message": "Download started",
            "download_url": plugin.download_url.clone()
        })))
    } else {
        Ok(HttpResponse::NotFound().json(serde_json::json!({
            "error": "Plugin not found"
        })))
    }
}

async fn rate_plugin(
    state: web::Data<AppState>,
    path: web::Path<String>,
    rating: web::Json<serde_json::Value>,
) -> Result<HttpResponse> {
    let plugin_id = path.into_inner();
    let mut plugins = state.plugins.lock().unwrap();
    
    if let Some(plugin) = plugins.get_mut(&plugin_id) {
        if let Some(rating_value) = rating.get("rating").and_then(|r| r.as_f64()) {
            if rating_value >= 0.0 && rating_value <= 5.0 {
                // Simple average rating calculation
                // In a real implementation, you'd store individual ratings
                plugin.rating = (plugin.rating + rating_value) / 2.0;
                
                Ok(HttpResponse::Ok().json(serde_json::json!({
                    "message": "Rating submitted successfully",
                    "new_rating": plugin.rating
                })))
            } else {
                Ok(HttpResponse::BadRequest().json(serde_json::json!({
                    "error": "Rating must be between 0.0 and 5.0"
                })))
            }
        } else {
            Ok(HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Invalid rating format"
            })))
        }
    } else {
        Ok(HttpResponse::NotFound().json(serde_json::json!({
            "error": "Plugin not found"
        })))
    }
}

async fn get_plugin_stats(state: web::Data<AppState>) -> Result<HttpResponse> {
    let plugins = state.plugins.lock().unwrap();
    
    let total_plugins = plugins.len();
    let total_downloads: u64 = plugins.values().map(|p| p.downloads).sum();
    let avg_rating: f64 = if total_plugins > 0 {
        plugins.values().map(|p| p.rating).sum::<f64>() / total_plugins as f64
    } else {
        0.0
    };
    
    let top_plugins: Vec<&PluginMetadata> = {
        let mut sorted: Vec<&PluginMetadata> = plugins.values().collect();
        sorted.sort_by(|a, b| b.downloads.cmp(&a.downloads));
        sorted.truncate(10)
    };
    
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "total_plugins": total_plugins,
        "total_downloads": total_downloads,
        "average_rating": avg_rating,
        "top_plugins": top_plugins
    })))
}

async fn health_check() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "timestamp": Utc::now()
    })))
}

fn load_sample_data() -> HashMap<String, PluginMetadata> {
    let mut plugins = HashMap::new();
    
    // Add some sample plugins
    let sample_plugins = vec![
        PluginMetadata {
            id: "echo-plugin".to_string(),
            name: "EchoPlugin".to_string(),
            version: "1.0.0".to_string(),
            description: "A simple echo plugin that returns the input as output".to_string(),
            author: "LAO Team".to_string(),
            license: "MIT".to_string(),
            repository: "https://github.com/lao-team/echo-plugin".to_string(),
            tags: vec!["basic".to_string(), "utility".to_string()],
            capabilities: vec![
                PluginCapability {
                    name: "echo".to_string(),
                    description: "Echo input as output".to_string(),
                    input_type: "text".to_string(),
                    output_type: "text".to_string(),
                }
            ],
            dependencies: vec![],
            downloads: 150,
            rating: 4.5,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            download_url: Some("https://github.com/lao-team/echo-plugin/releases/latest".to_string()),
            documentation_url: Some("https://github.com/lao-team/echo-plugin#readme".to_string()),
            compatible_versions: vec!["0.1.0".to_string(), "0.2.0".to_string()],
        },
        PluginMetadata {
            id: "ai-summarizer".to_string(),
            name: "AISummarizer".to_string(),
            version: "2.1.0".to_string(),
            description: "AI-powered text summarization plugin".to_string(),
            author: "AI Developer".to_string(),
            license: "Apache-2.0".to_string(),
            repository: "https://github.com/ai-dev/ai-summarizer".to_string(),
            tags: vec!["ai".to_string(), "nlp".to_string(), "summarization".to_string()],
            capabilities: vec![
                PluginCapability {
                    name: "summarize".to_string(),
                    description: "Summarize text using AI".to_string(),
                    input_type: "text".to_string(),
                    output_type: "text".to_string(),
                }
            ],
            dependencies: vec![
                PluginDependency {
                    name: "ollama".to_string(),
                    version: "0.1.0".to_string(),
                    optional: false,
                }
            ],
            downloads: 89,
            rating: 4.8,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            download_url: Some("https://github.com/ai-dev/ai-summarizer/releases/latest".to_string()),
            documentation_url: Some("https://github.com/ai-dev/ai-summarizer#readme".to_string()),
            compatible_versions: vec!["0.1.0".to_string(), "0.2.0".to_string()],
        },
    ];
    
    for plugin in sample_plugins {
        plugins.insert(plugin.id.clone(), plugin);
    }
    
    plugins
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    
    let app_state = web::Data::new(AppState {
        plugins: Mutex::new(load_sample_data()),
    });
    
    println!("🚀 LAO Plugin Registry Server starting...");
    println!("📡 API available at: http://localhost:8080");
    println!("🔍 Health check: http://localhost:8080/health");
    println!("📚 API docs: http://localhost:8080/plugins");
    
    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);
        
        App::new()
            .wrap(middleware::Logger::default())
            .wrap(cors)
            .app_data(app_state.clone())
            .route("/health", web::get().to(health_check))
            .route("/stats", web::get().to(get_plugin_stats))
            .route("/plugins", web::get().to(list_plugins))
            .route("/plugins", web::post().to(upload_plugin))
            .route("/plugins/{id}", web::get().to(get_plugin))
            .route("/plugins/{id}", web::put().to(update_plugin))
            .route("/plugins/{id}", web::delete().to(delete_plugin))
            .route("/plugins/{id}/download", web::post().to(download_plugin))
            .route("/plugins/{id}/rate", web::post().to(rate_plugin))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
} 