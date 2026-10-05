//! Catalogo de versiones descargables. Cada herramienta se consulta UNA vez y se guarda en
//! `cache/catalog.json`; solo `force = true` (boton "Actualizar") vuelve a la red.
use crate::error::{Ctx, Res};
use crate::{fsx, http, paths, tools::Tool};
use futures_util::{stream, StreamExt};
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Release {
    pub id: String,
    pub version: String,
    pub variant: Option<String>,
    pub url: String,
    pub size: Option<u64>,
    pub sha256: Option<String>,
}

impl Release {
    fn new(version: &str, variant: Option<&str>, url: String, size: Option<u64>, sha256: Option<String>) -> Self {
        let id = match variant {
            Some(v) => format!("{version}-{v}"),
            None => version.to_string(),
        };
        Release { id, version: version.into(), variant: variant.map(Into::into), url, size, sha256 }
    }
}

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct Cached {
    pub fetched_at: u64,
    pub releases: Vec<Release>,
}

#[derive(Default, Serialize, Deserialize)]
struct CatalogFile {
    tools: HashMap<String, Cached>,
}

pub async fn get(tool: Tool, force: bool) -> Res<Cached> {
    let mut file: CatalogFile = fsx::read_json(&paths::catalog_file());
    if !force {
        if let Some(c) = file.tools.get(tool.id()) {
            return Ok(c.clone());
        }
    }
    let mut releases = match tool {
        Tool::Php => php().await?,
        Tool::Apache => apache().await?,
        Tool::Mysql => mysql().await?,
        Tool::Mariadb => mariadb().await?,
        Tool::Composer => composer().await?,
        Tool::Phpmyadmin => phpmyadmin().await?,
        Tool::Postgres => postgres().await?,
        Tool::Heidisql => heidisql().await?,
    };
    releases.sort_by(|a, b| version_key(&b.version).cmp(&version_key(&a.version)).then(a.id.cmp(&b.id)));
    let cached = Cached { fetched_at: now(), releases };
    file.tools.insert(tool.id().into(), cached.clone());
    fsx::write_json(&paths::catalog_file(), &file)?;
    Ok(cached)
}

fn now() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

fn version_key(v: &str) -> Vec<u32> {
    v.split('.').map(|p| p.parse().unwrap_or(0)).collect()
}

fn push_unique(out: &mut Vec<Release>, rel: Release) {
    if !out.iter().any(|r| r.id == rel.id) {
        out.push(rel);
    }
}

async fn php() -> Res<Vec<Release>> {
    const BASE: &str = "https://downloads.php.net/~windows/releases";
    let mut out: Vec<Release> = Vec::new();

    // Ultimas de cada rama: traen sha256 y tamano.
    let latest = http::json(&format!("{BASE}/releases.json")).await?;
    let build = Regex::new(r"^(nts|ts)-v[cs]\d+-x64$").unwrap();
    for (_, branch) in latest.as_object().ctx("releases.json")? {
        let Some(version) = branch["version"].as_str() else { continue };
        for (key, val) in branch.as_object().into_iter().flatten() {
            let Some(caps) = build.captures(key) else { continue };
            let Some(path) = val["zip"]["path"].as_str() else { continue };
            let sha = val["zip"]["sha256"].as_str().map(Into::into);
            let url = format!("{BASE}/{path}");
            push_unique(&mut out, Release::new(version, Some(&caps[1]), url, parse_size(&val["zip"]["size"]), sha));
        }
    }

    // Historico: el listado de archives.
    let html = http::text(&format!("{BASE}/archives/"), None).await?;
    let file = Regex::new(r"php-(\d+\.\d+\.\d+)-(nts-)?Win32-v[cs]\d+-x64\.zip").unwrap();
    for caps in file.captures_iter(&html) {
        let variant = if caps.get(2).is_some() { "nts" } else { "ts" };
        let url = format!("{BASE}/archives/{}", &caps[0]);
        push_unique(&mut out, Release::new(&caps[1], Some(variant), url, None, None));
    }
    Ok(out)
}

fn parse_size(v: &Value) -> Option<u64> {
    let s = v.as_str()?;
    let n: f64 = s.trim_end_matches(|c: char| c.is_alphabetic()).trim().parse().ok()?;
    let mult = if s.ends_with("KB") { 1e3 } else if s.ends_with("GB") { 1e9 } else { 1e6 };
    Some((n * mult) as u64)
}

async fn composer() -> Res<Vec<Release>> {
    let v = http::json("https://getcomposer.org/versions").await?;
    let list = v["stable"].as_array().ctx("versions")?;
    Ok(list
        .iter()
        .filter_map(|e| {
            let version = e["version"].as_str()?;
            let path = e["path"].as_str()?;
            Some(Release::new(version, None, format!("https://getcomposer.org{path}"), None, None))
        })
        .collect())
}

async fn phpmyadmin() -> Res<Vec<Release>> {
    let html = http::text("https://www.phpmyadmin.net/files/", None).await?;
    let re = Regex::new(r"phpMyAdmin-(\d+\.\d+\.\d+)-all-languages\.zip").unwrap();
    let mut out: Vec<Release> = Vec::new();
    for caps in re.captures_iter(&html).filter(|c| version_key(&c[1])[0] >= 5) {
        let url = format!("https://files.phpmyadmin.net/phpMyAdmin/{0}/phpMyAdmin-{0}-all-languages.zip", &caps[1]);
        push_unique(&mut out, Release::new(&caps[1], None, url, None, None));
    }
    Ok(out)
}

async fn heidisql() -> Res<Vec<Release>> {
    let v = http::json("https://api.github.com/repos/HeidiSQL/HeidiSQL/releases?per_page=40").await?;
    let re = Regex::new(r"^HeidiSQL_([\d.]+)_64_Portable\.zip$").unwrap();
    let mut out: Vec<Release> = Vec::new();
    for asset in v.as_array().ctx("releases")?.iter().flat_map(|r| r["assets"].as_array().into_iter().flatten()) {
        let (Some(name), Some(url)) = (asset["name"].as_str(), asset["browser_download_url"].as_str()) else { continue };
        if let Some(caps) = re.captures(name) {
            push_unique(&mut out, Release::new(&caps[1], None, url.into(), asset["size"].as_u64(), None));
        }
    }
    Ok(out)
}

/// Binarios zip de EDB; sin API: HEAD sobre versiones conocidas (ampliar rangos aqui).
async fn postgres() -> Res<Vec<Release>> {
    let ranges: [(u32, std::ops::RangeInclusive<u32>); 6] =
        [(13, 0..=25), (14, 0..=25), (15, 0..=25), (16, 0..=20), (17, 0..=15), (18, 0..=10)];
    let candidates: Vec<String> = ranges.into_iter().flat_map(|(major, minors)| minors.map(move |m| format!("{major}.{m}"))).collect();
    let probed = stream::iter(candidates).map(|v| async move {
        let url = format!("https://get.enterprisedb.com/postgresql/postgresql-{v}-1-windows-x64-binaries.zip");
        let resp = http::client().head(&url).send().await.ok()?;
        let size = resp.headers().get("content-length").and_then(|h| h.to_str().ok()?.parse().ok());
        resp.status().is_success().then(|| Release::new(&v, None, url, size, None))
    });
    Ok(probed.buffer_unordered(12).collect::<Vec<_>>().await.into_iter().flatten().collect())
}

async fn apache() -> Res<Vec<Release>> {
    let html = http::text("https://www.apachelounge.com/download/", Tool::Apache.meta().referer).await?;
    let re = Regex::new(r"(?i)/download/VS\d+/binaries/httpd-(\d+\.\d+\.\d+)-\w+-Win64-VS\d+\.zip").unwrap();
    let mut out: Vec<Release> = Vec::new();
    for caps in re.captures_iter(&html) {
        let url = format!("https://www.apachelounge.com{}", &caps[0]);
        push_unique(&mut out, Release::new(&caps[1], None, url, None, None));
    }
    Ok(out)
}

async fn mariadb() -> Res<Vec<Release>> {
    const API: &str = "https://downloads.mariadb.org/rest-api/mariadb";
    let majors = http::json(&format!("{API}/")).await?;
    let ids: Vec<String> = majors["major_releases"]
        .as_array()
        .ctx("major_releases")?
        .iter()
        .filter(|m| m["release_status"] == "Stable")
        .filter_map(|m| m["release_id"].as_str().map(Into::into))
        .collect();
    let pages: Vec<Res<Value>> =
        stream::iter(ids).map(|id| async move { http::json(&format!("{API}/{id}/")).await }).buffer_unordered(4).collect().await;
    let mut out = Vec::new();
    for page in pages {
        for (version, rel) in page?["releases"].as_object().into_iter().flatten() {
            let zip = format!("mariadb-{version}-winx64.zip");
            let Some(f) = rel["files"].as_array().into_iter().flatten().find(|f| f["file_name"] == zip.as_str()) else {
                continue;
            };
            let url = format!("{API}/{version}/{zip}").replace("http://", "https://");
            out.push(Release::new(version, None, url, None, f["checksum"]["sha256sum"].as_str().map(Into::into)));
        }
    }
    Ok(out)
}

/// MySQL no tiene API publica: se prueban (HEAD) las versiones conocidas contra el CDN una sola vez.
async fn mysql() -> Res<Vec<Release>> {
    let ranges: [(&str, std::ops::RangeInclusive<u32>); 12] = [
        ("5.7", 30..=44),
        ("8.0", 20..=50),
        ("8.4", 0..=12),
        ("9.0", 0..=1),
        ("9.1", 0..=1),
        ("9.2", 0..=1),
        ("9.3", 0..=1),
        ("9.4", 0..=1),
        ("9.5", 0..=1),
        ("9.6", 0..=3),
        ("9.7", 0..=6),
        ("9.8", 0..=2),
    ];
    let candidates: Vec<(String, String)> = ranges
        .into_iter()
        .flat_map(|(series, patches)| patches.map(move |p| (series.to_string(), format!("{series}.{p}"))))
        .collect();
    let probed = stream::iter(candidates).map(|(series, v)| async move {
        let urls = [
            format!("https://cdn.mysql.com/Downloads/MySQL-{series}/mysql-{v}-winx64.zip"),
            format!("https://cdn.mysql.com/archives/mysql-{series}/mysql-{v}-winx64.zip"),
        ];
        for url in urls {
            if let Ok(resp) = http::client().head(&url).send().await {
                if resp.status().is_success() {
                    let size = resp.headers().get("content-length").and_then(|h| h.to_str().ok()?.parse().ok());
                    return Some(Release::new(&v, None, url, size, None));
                }
            }
        }
        None
    });
    Ok(probed.buffer_unordered(12).collect::<Vec<_>>().await.into_iter().flatten().collect())
}
