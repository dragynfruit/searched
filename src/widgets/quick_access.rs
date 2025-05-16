use base64::{Engine, engine::general_purpose::STANDARD};
use image::ImageFormat;
use once_cell::sync::Lazy;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::Cursor;

static SITE_ALIASES: Lazy<HashMap<&'static str, Vec<&'static str>>> = Lazy::new(|| {
    let mut m = HashMap::new();
    m.insert("discord", vec!["discord", "dsc"]);
    m.insert("github", vec!["github", "gh"]);
    m.insert("youtube", vec!["youtube", "yt"]);
    m.insert("twitter", vec!["twitter", "x", "tweet"]);
    m.insert("reddit", vec!["reddit", "rd"]);
    m.insert("gmail", vec!["gmail", "google mail"]);
    m.insert("outlook", vec!["outlook", "hotmail"]);
    m.insert("notion", vec!["notion"]);
    m.insert("slack", vec!["slack"]);
    m.insert(
        "microsoft teams",
        vec!["microsoft teams", "teams", "ms teams"],
    );
    m.insert("whatsapp", vec!["whatsapp", "wa"]);
    m.insert("telegram", vec!["telegram", "tg"]);
    m.insert("tiktok", vec!["tiktok", "tt"]);
    m.insert("pinterest", vec!["pinterest", "pin"]);
    m.insert("medium", vec!["medium"]);
    m.insert("trello", vec!["trello"]);
    m.insert("jira", vec!["jira", "atlassian"]);
    m.insert("figma", vec!["figma"]);
    m.insert("canva", vec!["canva"]);
    m.insert("zoom", vec!["zoom"]);
    m.insert("google docs", vec!["google docs", "gdocs", "docs"]);
    m
});

static SITES: Lazy<HashMap<&str, Site>> = Lazy::new(|| {
    let mut m = HashMap::new();
    m.insert("discord", Site {
        name: "Discord",
        url: "https://discord.com/app",
        icon: "https://assets-global.website-files.com/6257adef93867e50d84d30e2/636e0a6ca814282eca7172c6_icon_clyde_white_RGB.svg",
        color: "#5865F2",
    });
    m.insert(
        "github",
        Site {
            name: "GitHub",
            url: "https://github.com",
            icon: "https://github.githubassets.com/images/modules/logos_page/GitHub-Mark.png",
            color: "#171515",
        },
    );
    m.insert(
        "youtube",
        Site {
            name: "YouTube",
            url: "https://youtube.com",
            icon: "https://www.youtube.com/s/desktop/12d6b690/img/favicon_144x144.png",
            color: "#FF0000",
        },
    );
    m.insert(
        "twitter",
        Site {
            name: "Twitter",
            url: "https://twitter.com",
            icon: "https://abs.twimg.com/responsive-web/client-web/icon-ios.b1fc727a.png",
            color: "#1DA1F2",
        },
    );
    m.insert(
        "reddit",
        Site {
            name: "Reddit",
            url: "https://reddit.com",
            icon: "https://www.redditstatic.com/desktop2x/img/favicon/android-icon-192x192.png",
            color: "#FF4500",
        },
    );
    m.insert(
        "gmail",
        Site {
            name: "Gmail",
            url: "https://mail.google.com",
            icon: "https://www.gstatic.com/images/branding/product/2x/gmail_2020q4_32dp.png",
            color: "#EA4335",
        },
    );
    m.insert(
        "outlook",
        Site {
            name: "Outlook",
            url: "https://outlook.live.com",
            icon: "https://res.cdn.office.net/assets/mail/pwa/v1/pngs/outlook_base_48.png",
            color: "#0078D4",
        },
    );
    m.insert(
        "notion",
        Site {
            name: "Notion",
            url: "https://notion.so",
            icon: "https://www.notion.so/images/favicon.ico",
            color: "#000000",
        },
    );
    m.insert(
        "slack",
        Site {
            name: "Slack",
            url: "https://slack.com",
            icon: "https://a.slack-edge.com/80588/marketing/img/meta/slack_hash_256.png",
            color: "#4A154B",
        },
    );
    m.insert(
        "microsoft teams",
        Site {
            name: "Microsoft Teams",
            url: "https://teams.microsoft.com",
            icon:
                "https://statics.teams.cdn.office.net/evergreen-assets/apps/teams_shift_48x48.png",
            color: "#464EB8",
        },
    );
    m.insert(
        "whatsapp",
        Site {
            name: "WhatsApp",
            url: "https://web.whatsapp.com",
            icon: "https://web.whatsapp.com/img/favicon_c5088e888c97ad440a61d247596f88e5.png",
            color: "#25D366",
        },
    );
    m.insert(
        "telegram",
        Site {
            name: "Telegram",
            url: "https://web.telegram.org",
            icon: "https://telegram.org/img/website_icon.svg",
            color: "#0088cc",
        },
    );
    m.insert("tiktok", Site {
        name: "TikTok",
        url: "https://tiktok.com",
        icon: "https://sf16-scmcdn-va.ibytedtos.com/goofy/tiktok/web/node/_next/static/images/logo-black-10731.svg",
        color: "#000000",
    });
    m.insert(
        "pinterest",
        Site {
            name: "Pinterest",
            url: "https://pinterest.com",
            icon: "https://s.pinimg.com/webapp/favicon-54a5b2af.png",
            color: "#E60023",
        },
    );
    m.insert(
        "medium",
        Site {
            name: "Medium",
            url: "https://medium.com",
            icon: "https://cdn-static-1.medium.com/_/fp/icons/Medium-Avatar-500x500.svg",
            color: "#000000",
        },
    );
    m.insert(
        "trello",
        Site {
            name: "Trello",
            url: "https://trello.com",
            icon: "https://trello.com/favicon.ico",
            color: "#0079BF",
        },
    );
    m.insert(
        "jira",
        Site {
            name: "Jira",
            url: "https://jira.atlassian.com",
            icon: "https://wac-cdn.atlassian.com/assets/img/favicons/atlassian/favicon.png",
            color: "#0052CC",
        },
    );
    m.insert(
        "figma",
        Site {
            name: "Figma",
            url: "https://figma.com",
            icon: "https://static.figma.com/app/icon/1/favicon.svg",
            color: "#F24E1E",
        },
    );
    m.insert(
        "canva",
        Site {
            name: "Canva",
            url: "https://canva.com",
            icon: "https://static.canva.com/static/images/favicon-1.ico",
            color: "#00C4CC",
        },
    );
    m.insert(
        "zoom",
        Site {
            name: "Zoom",
            url: "https://zoom.us",
            icon: "https://st1.zoom.us/zoom.ico",
            color: "#2D8CFF",
        },
    );
    m.insert(
        "google docs",
        Site {
            name: "Google Docs",
            url: "https://docs.google.com",
            icon: "https://ssl.gstatic.com/docs/documents/images/kix-favicon7.ico",
            color: "#4285F4",
        },
    );
    // Add more sites as needed
    m
});

struct Site {
    name: &'static str,
    url: &'static str,
    icon: &'static str,
    color: &'static str,
}

#[derive(Debug, Serialize, Deserialize)]
struct CachedIcon {
    data_url: String,
    timestamp: u64,
}

#[derive(Debug, Serialize)]
pub struct QuickAccess {
    pub name: String,
    pub url: String,
    pub icon: String,
    pub color: String,
}

impl QuickAccess {
    pub async fn detect(query: &str, client: &Client, db: &sled::Db) -> Option<Self> {
        let query = query.trim().to_lowercase();

        // Find matching site through aliases
        for (site_key, aliases) in SITE_ALIASES.iter() {
            if aliases.iter().any(|&alias| query == alias.to_lowercase()) {
                if let Some(site) = SITES.get(site_key) {
                    return Some(Self::with_cached_icon(site, client, db).await);
                }
            }
        }
        None
    }

    async fn with_cached_icon(site: &Site, client: &Client, db: &sled::Db) -> Self {
        let icons = db.open_tree("site_icons").ok();
        let icon_key = format!("quickaccess_{}", site.url);

        // Try to get cached icon
        if let Some(tree) = &icons {
            if let Ok(Some(cached)) = tree.get(icon_key.as_bytes()) {
                if let Some(data_url) = Self::unpack_icon_data(&cached) {
                    return Self {
                        name: site.name.to_string(),
                        url: site.url.to_string(),
                        icon: data_url,
                        color: site.color.to_string(),
                    };
                }
            }
        }

        // Fetch and cache the icon
        let icon = match Self::fetch_and_cache_icon(site.icon, client, icons, &icon_key).await {
            Some(data_url) => data_url,
            None => site.icon.to_string(), // Fallback to original URL
        };

        Self {
            name: site.name.to_string(),
            url: site.url.to_string(),
            icon,
            color: site.color.to_string(),
        }
    }

    fn unpack_icon_data(packed: &[u8]) -> Option<String> {
        if let Ok(cached) = bincode::deserialize::<CachedIcon>(packed) {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .ok()?
                .as_secs();

            // Use cache if less than 7 days old
            if now - cached.timestamp < 7 * 24 * 60 * 60 {
                return Some(cached.data_url);
            }
        }
        None
    }

    async fn fetch_and_cache_icon(
        icon_url: &str,
        client: &Client,
        icons: Option<sled::Tree>,
        icon_key: &str,
    ) -> Option<String> {
        // Fetch icon
        let response = client.get(icon_url).send().await.ok()?;
        let bytes = response.bytes().await.ok()?;

        // Process image
        let img = image::load_from_memory(&bytes).ok()?;
        let resized = image::imageops::resize(&img, 32, 32, image::imageops::FilterType::Lanczos3);

        // Convert to PNG
        let mut png_data = Vec::new();
        resized
            .write_to(&mut Cursor::new(&mut png_data), ImageFormat::Png)
            .ok()?;

        // Convert to base64
        let base64 = STANDARD.encode(&png_data);
        let data_url = format!("data:image/png;base64,{}", base64);

        // Cache asynchronously
        if let Some(icons) = icons {
            let icons = icons.clone();
            let icon_key = icon_key.to_string();
            let data_url_clone = data_url.clone();

            tokio::spawn(async move {
                let cached = CachedIcon {
                    data_url: data_url_clone,
                    timestamp: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .ok()?
                        .as_secs(),
                };

                if let Ok(encoded) = bincode::serialize(&cached) {
                    let _ = icons.insert(icon_key.as_bytes(), encoded);
                }
                Some(())
            });
        }

        Some(data_url)
    }
}
