use std::env;
use std::sync::OnceLock;

/// User-facing strings. To add a language, add a `Strings` constant and match it in `for_locale`.
pub struct Strings {
    pub waiting: &'static str,
    pub launch_at_login: &'static str,
    pub quit: &'static str,
    pub listen_on_deezer: &'static str,
}

const EN: Strings = Strings {
    waiting: "Waiting for Deezer…",
    launch_at_login: "Launch at login",
    quit: "Quit",
    listen_on_deezer: "Listen on Deezer",
};

const FR: Strings = Strings {
    waiting: "En attente de Deezer…",
    launch_at_login: "Lancer au démarrage",
    quit: "Quitter",
    listen_on_deezer: "Écouter sur Deezer",
};

const ES: Strings = Strings {
    waiting: "Esperando a Deezer…",
    launch_at_login: "Abrir al iniciar sesión",
    quit: "Salir",
    listen_on_deezer: "Escuchar en Deezer",
};

const DE: Strings = Strings {
    waiting: "Warte auf Deezer…",
    launch_at_login: "Beim Anmelden starten",
    quit: "Beenden",
    listen_on_deezer: "Auf Deezer anhören",
};

const IT: Strings = Strings {
    waiting: "In attesa di Deezer…",
    launch_at_login: "Apri al login",
    quit: "Esci",
    listen_on_deezer: "Ascolta su Deezer",
};

const PT: Strings = Strings {
    waiting: "Aguardando o Deezer…",
    launch_at_login: "Abrir ao iniciar sessão",
    quit: "Sair",
    listen_on_deezer: "Ouvir no Deezer",
};

fn for_locale(locale: &str) -> &'static Strings {
    let lang = locale
        .split(['-', '_'])
        .next()
        .unwrap_or_default()
        .to_ascii_lowercase();
    match lang.as_str() {
        "fr" => &FR,
        "es" => &ES,
        "de" => &DE,
        "it" => &IT,
        "pt" => &PT,
        _ => &EN,
    }
}

/// Strings for `DEEZER_PRESENCE_LANG` if set, otherwise the system language, falling back to English.
pub fn t() -> &'static Strings {
    static STRINGS: OnceLock<&'static Strings> = OnceLock::new();
    STRINGS.get_or_init(|| {
        let locale = env::var("DEEZER_PRESENCE_LANG")
            .ok()
            .or_else(sys_locale::get_locale)
            .unwrap_or_default();
        for_locale(&locale)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn picks_language_from_locale() {
        assert_eq!(for_locale("fr-FR").quit, FR.quit);
        assert_eq!(for_locale("pt_BR").quit, PT.quit);
        assert_eq!(for_locale("DE").quit, DE.quit);
        assert_eq!(for_locale("ja-JP").quit, EN.quit);
        assert_eq!(for_locale("").quit, EN.quit);
    }
}
