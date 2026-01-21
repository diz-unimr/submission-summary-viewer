use sha2::{Digest, Sha256};
use std::fmt::Display;
use std::str::FromStr;

pub(crate) struct SubmissionSummary {
    pub(crate) tan: StringValue,
    pub(crate) code: StringValue,
    pub(crate) date: StringValue,
    pub(crate) counter: StringValue,
    pub(crate) ik: Ik,
    pub(crate) datacenter: Datacenter,
    pub(crate) typ_der_meldung: TypDerMeldung,
    pub(crate) indikationsbereich: Indikationsbereich,
    pub(crate) kostentraeger: Kostentraeger,
    pub(crate) art_der_daten: ArtDerDaten,
    pub(crate) art_der_sequenzierung: ArtDerSequenzierung,
    pub(crate) accepted: bool,
    pub(crate) hash_wert: StringValue,
    hash_string: String,
}

impl SubmissionSummary {
    pub(crate) fn valid_hash(&self) -> bool {
        let mut hasher = Sha256::new();
        hasher.update(self.hash_string.as_bytes());
        let hash_result = hasher.finalize();
        let hash_result = base16ct::lower::encode_string(&hash_result);
        hash_result == self.hash_wert.0
    }
}

impl SubmissionSummary {
    #[allow(clippy::expect_used)]
    fn matches_hash_tan_pattern(s: &str) -> bool {
        let regexp = regex::Regex::new(r"[0-9a-fA-F]{64}").expect("Invalid regexp");
        regexp.is_match(s)
    }

    #[allow(clippy::expect_used)]
    fn matches_count_pattern(s: &str) -> bool {
        let regexp = regex::Regex::new(r"^[0-9]{3}$").expect("Invalid regexp");
        regexp.is_match(s)
    }

    #[allow(clippy::expect_used)]
    fn is_reasonable_date(s: &str) -> bool {
        let regexp = regex::Regex::new(r"^20[0-9]{2}-(0[1-9]|1[0-2])-([0-2][0-9]|3[0-1])$")
            .expect("Invalid regexp");
        regexp.is_match(s)
    }

    #[allow(clippy::expect_used)]
    fn parse_date_and_number(s: &str) -> Option<(String, String)> {
        let regexp =
            regex::Regex::new(r"^20[0-9]{2}(0[1-9]|1[0-2])([0-2][0-9]|3[0-1])[0-9]{0,2}[1-9]$")
                .expect("Invalid regexp");

        if s.len() < 9 || !regexp.is_match(s) {
            return None;
        }

        let date = format!("{}-{}-{}", &s[0..4], &s[4..6], &s[6..8]);
        let counter = s[8..s.len()].parse::<String>().ok();

        counter.map(|counter| (date, counter))
    }
}

impl FromStr for SubmissionSummary {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts = s.lines().collect::<Vec<&str>>();
        if parts.len() < 2 || parts[0] != "Vorgangsnummer,Meldebestaetigung" {
            return Err(());
        }

        let parts = parts[1].trim().split(',').collect::<Vec<&str>>();
        if parts.len() != 2 {
            return Err(());
        }

        let tan = parts[0].to_string();

        let parts = parts[1].split('+').collect::<Vec<&str>>();
        if parts.len() != 5 || parts[0] != "IBE" {
            return Err(());
        }
        let hash_string = parts[2].to_string();
        let hash_wert = parts[4].to_string();

        let parts = parts[2].split('&').collect::<Vec<&str>>();
        if parts.len() != 11 {
            return Err(());
        }

        let Some((date, counter)) = Self::parse_date_and_number(parts[1]) else {
            return Err(());
        };

        Ok(SubmissionSummary {
            tan: StringValue::new(&tan, !Self::matches_hash_tan_pattern(&tan)),
            code: StringValue::new_valid(parts[0]),
            date: StringValue::new(&date, !Self::is_reasonable_date(&date)),
            counter: StringValue::new(&counter, !Self::matches_count_pattern(&counter)),
            ik: parts[2].to_string().parse()?,
            datacenter: parts[3].to_string().parse()?,
            typ_der_meldung: parts[4].to_string().parse()?,
            indikationsbereich: parts[5].to_string().parse()?,
            kostentraeger: parts[7].to_string().parse()?,
            art_der_daten: parts[8].to_string().parse()?,
            art_der_sequenzierung: parts[9].to_string().parse()?,
            accepted: parts[10] == "1",
            hash_string,
            hash_wert: StringValue::new(&hash_wert, !Self::matches_hash_tan_pattern(&hash_wert)),
        })
    }
}

pub(crate) trait CheckedValue
where
    Self: Display + Sized,
{
    fn is_invalid(&self) -> bool;
}

pub(crate) struct StringValue(String, bool);

#[allow(unused)]
impl StringValue {
    pub fn new(s: &str, invalid: bool) -> Self {
        Self(s.to_string(), invalid)
    }

    pub fn new_valid(s: &str) -> Self {
        Self::new(s, false)
    }

    pub fn new_invalid(s: &str) -> Self {
        Self::new(s, true)
    }
}

impl CheckedValue for StringValue {
    fn is_invalid(&self) -> bool {
        self.1 || self.0.is_empty()
    }
}

impl Display for StringValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, PartialEq)]
pub(crate) enum Datacenter {
    GRZK00001,
    GRZTUE002,
    GRZHD0003,
    GRZDD0004,
    GRZM00006,
    GRZB00007,
    KDKDD0001,
    KDKTUE002,
    KDKL00003,
    KDKL00004,
    KDKTUE005,
    KDKHD0006,
    KDKK00007,
    Unknown(String),
}

impl CheckedValue for Datacenter {
    fn is_invalid(&self) -> bool {
        matches!(self, Self::Unknown(_))
    }
}

impl FromStr for Datacenter {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "GRZK00001" => Ok(Datacenter::GRZK00001),
            "GRZTUE002" => Ok(Datacenter::GRZTUE002),
            "GRZHD0003" => Ok(Datacenter::GRZHD0003),
            "GRZDD0004" => Ok(Datacenter::GRZDD0004),
            "GRZM00006" => Ok(Datacenter::GRZM00006),
            "GRZB00007" => Ok(Datacenter::GRZB00007),
            "KDKDD0001" => Ok(Datacenter::KDKDD0001),
            "KDKTUE002" => Ok(Datacenter::KDKTUE002),
            "KDKL00003" => Ok(Datacenter::KDKL00003),
            "KDKL00004" => Ok(Datacenter::KDKL00004),
            "KDKTUE005" => Ok(Datacenter::KDKTUE005),
            "KDKHD0006" => Ok(Datacenter::KDKHD0006),
            "KDKK00007" => Ok(Datacenter::KDKK00007),
            u => Ok(Datacenter::Unknown(u.to_string())),
        }
    }
}

impl Display for Datacenter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Datacenter::GRZK00001 => write!(f, "GRZ Köln (GRZK00001)"),
            Datacenter::GRZTUE002 => write!(f, "GRZ Tübingen (GRZTUE002)"),
            Datacenter::GRZHD0003 => write!(f, "GRZ Heidelberg (GRZHD0003)"),
            Datacenter::GRZDD0004 => write!(f, "GRZ Dresden (GRZDD0004)"),
            Datacenter::GRZM00006 => write!(f, "GRZ München (GRZM00006)"),
            Datacenter::GRZB00007 => write!(f, "GRZ Berlin (GRZB00007)"),
            Datacenter::KDKDD0001 => write!(f, "Gfh-NET (KDKDD0001)"),
            Datacenter::KDKTUE002 => write!(f, "NSE (KDKTUE002)"),
            Datacenter::KDKL00003 => write!(f, "DK-FBREK (KDKL00003)"),
            Datacenter::KDKL00004 => write!(f, "DK-FDK (KDKL00004)"),
            Datacenter::KDKTUE005 => write!(f, "DNPM (KDKTUE005)"),
            Datacenter::KDKHD0006 => write!(f, "NCT/DKTK MASTER (KDKHD0006)"),
            Datacenter::KDKK00007 => write!(f, "nNGM (KDKK00007)"),
            Datacenter::Unknown(u) => write!(f, "Unbekannter Wert: '{u}'"),
        }
    }
}

#[derive(Debug, PartialEq)]
pub(crate) enum Ik {
    Ik260530012,
    Ik261101015,
    Ik260590071,
    Ik260530103,
    Ik261401030,
    Ik260510018,
    Ik260950567,
    Ik260510381,
    Ik260832299,
    Ik260610279,
    Ik260310378,
    Ik261500702,
    Ik260200013,
    Ik260320597,
    Ik260820466,
    Ik261600736,
    Ik260530283,
    Ik261401052,
    Ik260730161,
    Ik260620431, // Marburg
    Ik260914050,
    Ik260913195,
    Ik260550131,
    Ik260930608,
    Ik260102343,
    Ik260840108,
    Ik260840200,
    Ik260960079,
    Unknown(String),
}

impl CheckedValue for Ik {
    fn is_invalid(&self) -> bool {
        matches!(self, Self::Unknown(_))
    }
}

impl FromStr for Ik {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "260530012" => Ok(Ik::Ik260530012),
            "261101015" => Ok(Ik::Ik261101015),
            "260590071" => Ok(Ik::Ik260590071),
            "260530103" => Ok(Ik::Ik260530103),
            "261401030" => Ok(Ik::Ik261401030),
            "260510018" => Ok(Ik::Ik260510018),
            "260950567" => Ok(Ik::Ik260950567),
            "260510381" => Ok(Ik::Ik260510381),
            "260832299" => Ok(Ik::Ik260832299),
            "260610279" => Ok(Ik::Ik260610279),
            "260310378" => Ok(Ik::Ik260310378),
            "261500702" => Ok(Ik::Ik261500702),
            "260200013" => Ok(Ik::Ik260200013),
            "260320597" => Ok(Ik::Ik260320597),
            "260820466" => Ok(Ik::Ik260820466),
            "261600736" => Ok(Ik::Ik261600736),
            "260530283" => Ok(Ik::Ik260530283),
            "261401052" => Ok(Ik::Ik261401052),
            "260730161" => Ok(Ik::Ik260730161),
            "260620431" => Ok(Ik::Ik260620431),
            "260914050" => Ok(Ik::Ik260914050),
            "260913195" => Ok(Ik::Ik260913195),
            "260550131" => Ok(Ik::Ik260550131),
            "260930608" => Ok(Ik::Ik260930608),
            "260102343" => Ok(Ik::Ik260102343),
            "260840108" => Ok(Ik::Ik260840108),
            "260840200" => Ok(Ik::Ik260840200),
            "260960079" => Ok(Ik::Ik260960079),
            u => Ok(Ik::Unknown(u.to_string())),
        }
    }
}

impl Display for Ik {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Ik::Ik260530012 => write!(f, "Universitätsklinikum Aachen (260530012)"),
            Ik::Ik261101015 => write!(f, "Charité Universitätsmedizin Berlin (261101015)"),
            Ik::Ik260590071 => write!(
                f,
                "Universitätsklinikum der Ruhr-Universität Bochum (260590071)"
            ),
            Ik::Ik261401030 => write!(
                f,
                "Universitätsklinikum Carl Gustav Carus an der TU Dresden (261401030)"
            ),
            Ik::Ik260530103 => write!(f, "Universitätsklinikum Bonn (260530103)"),
            Ik::Ik260510018 => write!(f, "Universitätsklinikum Düsseldorf (260510018)"),
            Ik::Ik260950567 => write!(f, "Universitätsklinikum Erlangen (260950567)"),
            Ik::Ik260510381 => write!(f, "Universitätsklinikum Essen (260510381)"),
            Ik::Ik260832299 => write!(f, "Universitätsklinikum Freiburg (260832299)"),
            Ik::Ik260610279 => write!(
                f,
                "Universitätsklinikum Gießen und Marburg, Standort Gießen (260610279)"
            ),
            Ik::Ik260310378 => write!(f, "Universitätsmedizin Göttingen (260310378)"),
            Ik::Ik261500702 => write!(f, "Universitätsklinikum Halle (261500702)"),
            Ik::Ik260200013 => write!(f, "Universitätsklinikum Hamburg-Eppendorf (260200013)"),
            Ik::Ik260320597 => write!(f, "Medizinische Hochschule Hannover (260320597)"),
            Ik::Ik260820466 => write!(f, "Universitätsklinikum Heidelberg (260820466)"),
            Ik::Ik261600736 => write!(f, "Universitätsklinikum Jena (261600736)"),
            Ik::Ik260530283 => write!(f, "Universitätsklinikum Köln (260530283)"),
            Ik::Ik261401052 => write!(f, "Universitätsklinikum Leipzig (261401052)"),
            Ik::Ik260730161 => write!(f, "Universitätsmedizin Mainz (260730161)"),
            Ik::Ik260620431 => write!(
                f,
                "Universitätsklinikum Gießen und Marburg, Standort Marburg (260620431)"
            ),
            Ik::Ik260914050 => write!(f, "Klinikum der Universität München (260914050)"),
            Ik::Ik260913195 => write!(
                f,
                "Klinikum rechts der Isar der TU München/TUM-Klinikum (260913195)"
            ),
            Ik::Ik260550131 => write!(f, "Universitätsklinikum Münster (260550131)"),
            Ik::Ik260930608 => write!(f, "Universitätsklinikum Regensburg (260930608)"),
            Ik::Ik260102343 => write!(f, "Universitätsklinikum Schleswig-Holstein (260102343)"),
            Ik::Ik260840108 => write!(f, "Universitätsklinikum Tübingen (260840108)"),
            Ik::Ik260840200 => write!(f, "Universitätsklinikum Ulm (260840200)"),
            Ik::Ik260960079 => write!(f, "Universitätsklinikum Würzburg (260960079)"),
            Ik::Unknown(u) => write!(f, "Unbekannter Wert: '{u}'"),
        }
    }
}

#[derive(Debug, PartialEq)]
pub(crate) enum TypDerMeldung {
    Erstmeldung,
    FollowUp,
    Nachmeldung,
    Korrektur,
    Testmeldung,
    Unknown(String),
}

impl CheckedValue for TypDerMeldung {
    fn is_invalid(&self) -> bool {
        matches!(self, Self::Unknown(_))
    }
}

impl FromStr for TypDerMeldung {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "0" => Ok(TypDerMeldung::Erstmeldung),
            "1" => Ok(TypDerMeldung::FollowUp),
            "2" => Ok(TypDerMeldung::Nachmeldung),
            "3" => Ok(TypDerMeldung::Korrektur),
            "9" => Ok(TypDerMeldung::Testmeldung),
            u => Ok(TypDerMeldung::Unknown(u.to_string())),
        }
    }
}

impl Display for TypDerMeldung {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TypDerMeldung::Erstmeldung => write!(f, "Erstmeldung"),
            TypDerMeldung::FollowUp => write!(f, "Follow-Up"),
            TypDerMeldung::Nachmeldung => write!(f, "Nachmeldung"),
            TypDerMeldung::Korrektur => write!(f, "Korrektur"),
            TypDerMeldung::Testmeldung => write!(f, "Testmeldung"),
            TypDerMeldung::Unknown(u) => write!(f, "Unbekannter Wert: '{u}'"),
        }
    }
}

#[derive(Debug, PartialEq)]
pub(crate) enum Indikationsbereich {
    O,
    R,
    H,
    Unknown(String),
}

impl CheckedValue for Indikationsbereich {
    fn is_invalid(&self) -> bool {
        matches!(self, Self::Unknown(_))
    }
}

impl FromStr for Indikationsbereich {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "O" => Ok(Indikationsbereich::O),
            "R" => Ok(Indikationsbereich::R),
            "H" => Ok(Indikationsbereich::H),
            u => Ok(Indikationsbereich::Unknown(u.to_string())),
        }
    }
}

impl Display for Indikationsbereich {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Indikationsbereich::O => write!(f, "Onkologische Erkrankung"),
            Indikationsbereich::R => write!(f, "Seltene Erkrankung"),
            Indikationsbereich::H => write!(f, "Hereditäres Tumorprädispositionssyndrom"),
            Indikationsbereich::Unknown(u) => write!(f, "Unbekannter Wert: '{u}'"),
        }
    }
}

#[derive(Debug, PartialEq)]
pub(crate) enum Kostentraeger {
    Gkv,
    Pkv,
    PkvBeihilfe,
    Andere,
    Unknown(String),
}

impl CheckedValue for Kostentraeger {
    fn is_invalid(&self) -> bool {
        matches!(self, Self::Unknown(_))
    }
}

impl FromStr for Kostentraeger {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "1" => Ok(Kostentraeger::Gkv),
            "2" => Ok(Kostentraeger::Pkv),
            "3" => Ok(Kostentraeger::PkvBeihilfe),
            "4" => Ok(Kostentraeger::Andere),
            u => Ok(Kostentraeger::Unknown(u.to_string())),
        }
    }
}

impl Display for Kostentraeger {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Kostentraeger::Gkv => write!(f, "GKV"),
            Kostentraeger::Pkv => write!(f, "PKV"),
            Kostentraeger::PkvBeihilfe => write!(f, "PKV/Beihilfe"),
            Kostentraeger::Andere => write!(f, "andere"),
            Kostentraeger::Unknown(u) => write!(f, "Unbekannter Wert: '{u}'"),
        }
    }
}

#[derive(Debug, PartialEq)]
pub(crate) enum ArtDerDaten {
    C,
    G,
    Unknown(String),
}

impl CheckedValue for ArtDerDaten {
    fn is_invalid(&self) -> bool {
        matches!(self, Self::Unknown(_))
    }
}

impl FromStr for ArtDerDaten {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "C" => Ok(ArtDerDaten::C),
            "G" => Ok(ArtDerDaten::G),
            u => Ok(ArtDerDaten::Unknown(u.to_string())),
        }
    }
}

impl Display for ArtDerDaten {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ArtDerDaten::C => write!(f, "Klinische Daten"),
            ArtDerDaten::G => write!(f, "genomische Daten"),
            ArtDerDaten::Unknown(u) => write!(f, "Unbekannter Wert: '{u}'"),
        }
    }
}

#[derive(Debug, PartialEq)]
pub(crate) enum ArtDerSequenzierung {
    Keine,
    Wgs,
    Wes,
    Panel,
    WgsLr,
    Unknown(String),
}

impl CheckedValue for ArtDerSequenzierung {
    fn is_invalid(&self) -> bool {
        matches!(self, Self::Unknown(_))
    }
}

impl FromStr for ArtDerSequenzierung {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "0" => Ok(ArtDerSequenzierung::Keine),
            "1" => Ok(ArtDerSequenzierung::Wgs),
            "2" => Ok(ArtDerSequenzierung::Wes),
            "3" => Ok(ArtDerSequenzierung::Panel),
            "4" => Ok(ArtDerSequenzierung::WgsLr),
            u => Ok(ArtDerSequenzierung::Unknown(u.to_string())),
        }
    }
}

impl Display for ArtDerSequenzierung {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ArtDerSequenzierung::Keine => write!(f, "Keine"),
            ArtDerSequenzierung::Wgs => write!(f, "WGS"),
            ArtDerSequenzierung::Wes => write!(f, "WES"),
            ArtDerSequenzierung::Panel => write!(f, "Panel"),
            ArtDerSequenzierung::WgsLr => write!(f, "WGS/LR"),
            ArtDerSequenzierung::Unknown(u) => write!(f, "Unbekannter Wert: '{u}'"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_parse() {
        let parsed = SubmissionSummary::from_str("Vorgangsnummer,Meldebestaetigung\nbad8a31b1759b565bee3d283e68af38e173499bfcce2f50691e7eddda62b2f31,IBE+A123456789+A123456789&20240701001&260530103&KDKK00001&0&O&9&1&C&2&1+9+bad8a31b1759b565bee3d283e68af38e173499bfcce2f50691e7eddda62b2f31").unwrap();

        assert_eq!(
            parsed.tan.to_string(),
            "bad8a31b1759b565bee3d283e68af38e173499bfcce2f50691e7eddda62b2f31"
        );
        assert_eq!(parsed.code.to_string(), "A123456789");
        assert_eq!(parsed.date.to_string(), "2024-07-01");
        assert_eq!(parsed.counter.to_string(), "001");
        assert_eq!(parsed.ik, Ik::Ik260530103);
        assert_eq!(
            parsed.datacenter,
            Datacenter::Unknown("KDKK00001".to_string())
        );
        assert_eq!(parsed.typ_der_meldung, TypDerMeldung::Erstmeldung);
        assert_eq!(parsed.indikationsbereich, Indikationsbereich::O);
        assert_eq!(parsed.kostentraeger, Kostentraeger::Gkv);
        assert_eq!(parsed.art_der_daten, ArtDerDaten::C);
        assert_eq!(parsed.art_der_sequenzierung, ArtDerSequenzierung::Wes);
        assert!(parsed.accepted);
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_valid_hash_validation() {
        let parsed = SubmissionSummary::from_str("Vorgangsnummer,Meldebestaetigung\nbad8a31b1759b565bee3d283e68af38e173499bfcce2f50691e7eddda62b2f31,IBE+A123456789+A123456789&20240701001&260530103&KDKK00001&0&O&9&1&C&2&1+9+bad8a31b1759b565bee3d283e68af38e173499bfcce2f50691e7eddda62b2f31").unwrap();
        assert!(parsed.valid_hash());
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_invalid_hash_validation() {
        let parsed = SubmissionSummary::from_str("Vorgangsnummer,Meldebestaetigung\nbad8a31b1759b565bee3d283e68af38e173499bfcce2f50691e7eddda62b2f31,IBE+A999999999+A999999999&20240701001&260530103&KDKK00001&0&O&9&1&C&2&1+9+bad8a31b1759b565bee3d283e68af38e173499bfcce2f50691e7eddda62b2f31").unwrap();
        assert!(!parsed.valid_hash());
    }

    #[rstest]
    #[case("2026-01-01", true)]
    #[case("1800-01-01", false)]
    #[case("2100-01-01", false)]
    #[case("2026-23-35", false)]
    #[case("2026-1-1", false)]
    #[case("2026-01-10", true)]
    #[case("2026-10-12", true)]
    #[case("2024-02-29", true)]
    #[case("2026-13-01", false)]
    #[case("2026-01-32", false)]
    fn test_date_validation(#[case] date: &str, #[case] expected: bool) {
        assert_eq!(SubmissionSummary::is_reasonable_date(date), expected);
    }

    #[rstest]
    #[case("1", false)]
    #[case("001", true)]
    #[case("100", true)]
    #[case("123", true)]
    #[case("11", false)]
    #[case("1234", false)]
    fn test_number_validation(#[case] date: &str, #[case] expected: bool) {
        assert_eq!(SubmissionSummary::matches_count_pattern(date), expected);
    }

    #[rstest]
    #[case("202601011", "2026-01-01", "1")]
    #[case("2026010112", "2026-01-01", "12")]
    #[case("20260101123", "2026-01-01", "123")]
    #[case("20260109001", "2026-01-09", "001")]
    #[case("20260110001", "2026-01-10", "001")]
    #[case("20260111123", "2026-01-11", "123")]
    #[case("20260919123", "2026-09-19", "123")]
    #[case("20261020123", "2026-10-20", "123")]
    #[case("20261221123", "2026-12-21", "123")]
    #[case("20261229123", "2026-12-29", "123")]
    #[case("20261230123", "2026-12-30", "123")]
    #[case("20261231123", "2026-12-31", "123")]
    fn test_should_parse_date_and_number(
        #[case] input: &str,
        #[case] date: &str,
        #[case] number: &str,
    ) {
        assert_eq!(
            SubmissionSummary::parse_date_and_number(input),
            Some((date.to_string(), number.to_string()))
        );
    }

    #[rstest]
    fn test_should_not_parse_date_and_number(
        #[values("20260101", "20260101123456789", "260101001", "", "irgendwas")] input: &str,
    ) {
        assert_eq!(SubmissionSummary::parse_date_and_number(input), None);
    }
}
