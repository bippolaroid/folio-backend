use serde::{Deserialize, Serialize};
use serde_json::Number;

#[derive(Serialize, Deserialize, Clone)]
pub struct Keypoint {
    pub id: i32,
    pub featured: Vec<String>,
    pub title: String,
    pub summary: String,
}

#[derive(Deserialize, Clone)]
pub struct ProjectData {
    pub id: Number,
    pub client: String,
    pub client_logo: String,
    pub accent_color: String,
    pub title: String,
    pub tags: Vec<String>,
    pub featured: String,
    pub keypoints: Vec<Keypoint>,
}

#[derive(Serialize)]
pub struct Project {
    pub id: Number,
    pub client: String,
    pub client_logo: String,
    pub accent_color: String,
    pub title: String,
    pub tags: Vec<String>,
    pub featured: String,
    pub keypoints: Vec<Keypoint>,
}