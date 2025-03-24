// SPDX-License-Identifier: Apache-2.0

use std::fmt::{
    self,
    Debug,
    Display,
    Formatter,
};
use std::str::FromStr;

use tinystr::TinyAsciiStr;

use crate::ethereum::SolidityAddress;
use crate::ledger_id::RefLedgerId;
use crate::{
    Client,
    Error,
};

#[derive(Hash, PartialEq, Eq, Clone, Copy)]
pub struct Checksum(TinyAsciiStr<5>);

impl Checksum {
    fn from_bytes(bytes: [u8; 5]) -> Checksum {
        Checksum(TinyAsciiStr::from_bytes(&bytes).unwrap())
    }
}

impl FromStr for Checksum {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let ascii_str: TinyAsciiStr<5> = s
            .parse()
            .map_err(|e| Error::basic_parse(format!("Expected checksum to be valid ascii: {e}")))?;

        if ascii_str.len() != 5 || !ascii_str.is_ascii_alphabetic_lowercase() {
            return Err(Error::basic_parse("Expected checksum to be exactly 5 lowercase letters"));
        }

        Ok(Self(ascii_str))
    }
}

impl Display for Checksum {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(self.0.as_str())
    }
}

impl Debug for Checksum {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "\"{}\"", self.0.as_str())
    }
}

pub trait ValidateChecksums {
    /// Validates all entity-id checksums for `self` with the given ledger-id.
    ///
    /// # Errors
    /// - [`Error::BadEntityId`] if any of the expected checksums don't match the actual checksums.
    fn validate_checksums(&self, ledger_id: &RefLedgerId) -> crate::Result<()>;
}

impl<T: ValidateChecksums> ValidateChecksums for Option<T> {
    fn validate_checksums(&self, ledger_id: &RefLedgerId) -> crate::Result<()> {
        if let Some(id) = &self {
            id.validate_checksums(ledger_id)?;
        }
        Ok(())
    }
}

/// The ID of an entity on the Hedera network.
#[derive(Hash, PartialEq, Eq, Clone, Copy)]
pub struct EntityId {
    /// A non-negative number identifying the shard containing this entity.
    pub shard: u64,

    /// A non-negative number identifying the realm within the shard containing this entity.
    pub realm: u64,

    /// A non-negative number identifying the entity within the realm containing this entity.
    pub num: u64,

    /// A checksum if the entity ID was read from a user inputted string which inclueded a checksum
    pub checksum: Option<Checksum>,
}

#[derive(Copy, Clone)]
pub(crate) enum PartialEntityId<'a> {
    ShortNum(u64),
    LongNum(EntityId),
    ShortOther(&'a str),
    LongOther { shard: u64, realm: u64, last: &'a str },
}

impl<'a> PartialEntityId<'a> {
    pub(crate) fn finish<T>(self) -> crate::Result<T>
    where
        EntityId: Into<T>,
    {
        match self {
            Self::ShortNum(num) => Ok(EntityId::from(num).into()),
            Self::LongNum(id) => Ok(id.into()),
            _ => Err(Error::basic_parse("expected `<shard>.<realm>.<num>`".to_owned())),
        }
    }

    // `FromStr` doesn't allow lifetime bounds.
    pub(crate) fn from_str(s: &'a str) -> crate::Result<Self> {
        let expecting =
            || Error::basic_parse(format!("expected `<shard>.<realm>.<num>`, got `{s}`"));

        // entity ID parsing is painful because there are 4 formats...
        // This way avoids allocations at the code of an extra layer of nesting.
        match s.split_once('.') {
            Some((shard, rest)) => {
                let (realm, last) = rest.split_once('.').ok_or_else(expecting)?;

                if last.is_empty() {
                    return Err(expecting());
                }

                let shard = shard.parse().map_err(|_| expecting())?;
                let realm = realm.parse().map_err(|_| expecting())?;

                match last.rsplit_once('-') {
                    Some((num, checksum)) => {
                        let num = num.parse().map_err(|_| expecting())?;
                        let checksum = Some(checksum.parse()?);

                        Ok(Self::LongNum(EntityId { shard, realm, num, checksum }))
                    }

                    None => match last.parse() {
                        Ok(num) => {
                            Ok(Self::LongNum(EntityId { shard, realm, num, checksum: None }))
                        }

                        Err(_) => Ok(Self::LongOther { shard, realm, last }),
                    },
                }
            }
            None => match s.parse() {
                Ok(it) => return Ok(Self::ShortNum(it)),
                Err(_) => return Ok(Self::ShortOther(s)),
            },
        }
    }
}

impl EntityId {
    /// Parse an entity ID from a solidity address
    ///
    /// # Errors
    /// - [`Error::BasicParse`] if `address` cannot be parsed as a solidity address.
    pub(crate) fn from_solidity_address(address: &str) -> crate::Result<Self> {
        SolidityAddress::from_str(address).map(Self::from)
    }

    /// Convert `self` into a solidity `address`.
    ///
    /// # Errors
    /// - [`Error::BasicParse`] if `self.shard` is larger than `u32::MAX`.
    pub(crate) fn to_solidity_address(self) -> crate::Result<String> {
        SolidityAddress::try_from(self).map(|it| it.to_string())
    }

    pub(crate) fn generate_checksum(entity_id_string: &str, ledger_id: &RefLedgerId) -> Checksum {
        const P3: usize = 26 * 26 * 26; // 3 digits in base 26
        const P5: usize = 26 * 26 * 26 * 26 * 26; // 5 digits in base 26
        const M: usize = 1_000_003; // min prime greater than a million. Used for the final permutation.
        const W: usize = 31; // Sum s of digit values weights them by powers of W. Should be coprime to P5.

        let h = [ledger_id.as_bytes().to_vec(), vec![0u8; 6]].concat();

        // Digits with 10 for ".", so if addr == "0.0.123" then d == [0, 10, 0, 10, 1, 2, 3]
        let d = entity_id_string.chars().map(|c| {
            if c == '.' {
                10_usize
            } else {
                c.to_digit(10).unwrap() as usize
            }
        });

        let mut s = 0; // Weighted sum of all positions (mod P3)
        let mut s0 = 0; // Sum of even positions (mod 11)
        let mut s1 = 0; // Sum of odd positions (mod 11)
        for (i, digit) in d.enumerate() {
            s = (W * s + digit) % P3;
            if i % 2 == 0 {
                s0 = (s0 + digit) % 11;
            } else {
                s1 = (s1 + digit) % 11;
            }
        }

        let mut sh = 0; // Hash of the ledger ID
        for b in h {
            sh = (W * sh + (b as usize)) % P5;
        }

        // The checksum, as a single number
        let mut c = ((((entity_id_string.len() % 5) * 11 + s0) * 11 + s1) * P3 + s + sh) % P5;
        c = (c * M) % P5;

        let mut answer = [0_u8; 5];
        for i in (0..5).rev() {
            answer[i] = b'a' + ((c % 26) as u8);
            c /= 26;
        }

        Checksum::from_bytes(answer)
    }

    /// Validates that the the checksum computed for the given `shard.realm.num` matches the given checksum.
    ///
    /// # Errors
    /// - [`Error::CannotPerformTaskWithoutLedgerId`] if the client has no `ledger_id`.
    /// - [`Error::BadEntityId`] if there is a checksum, and the checksum is not valid for the client's `ledger_id`.
    pub(crate) fn validate_checksum(
        shard: u64,
        realm: u64,
        num: u64,
        checksum: Option<Checksum>,
        client: &Client,
    ) -> crate::Result<()> {
        let Some(present_checksum) = checksum else {
            return Ok(());
        };

        let ledger_id = client.ledger_id_internal();
        let ledger_id = ledger_id
            .as_deref()
            .expect("Client had no ledger ID (help: call `client.set_ledger_id()`");

        Self::validate_checksum_internal(
            shard,
            realm,
            num,
            present_checksum,
            ledger_id.as_ref_ledger_id(),
        )
    }

    pub(crate) fn validate_checksum_for_ledger_id(
        shard: u64,
        realm: u64,
        num: u64,
        checksum: Option<Checksum>,
        ledger_id: &RefLedgerId,
    ) -> Result<(), Error> {
        if let Some(present_checksum) = checksum {
            Self::validate_checksum_internal(shard, realm, num, present_checksum, ledger_id)
        } else {
            Ok(())
        }
    }

    fn validate_checksum_internal(
        shard: u64,
        realm: u64,
        num: u64,
        present_checksum: Checksum,
        ledger_id: &RefLedgerId,
    ) -> Result<(), Error> {
        let expected_checksum =
            Self::generate_checksum(&format!("{shard}.{realm}.{num}"), ledger_id);
        if present_checksum == expected_checksum {
            Ok(())
        } else {
            Err(Error::BadEntityId { shard, realm, num, present_checksum, expected_checksum })
        }
    }

    pub(crate) fn to_string_with_checksum(mut entity_id_string: String, client: &Client) -> String {
        let ledger_id = client.ledger_id_internal();
        let ledger_id = ledger_id
            .as_ref()
            .expect("Client had no ledger ID (help: call `client.set_ledger_id()`");

        let checksum = Self::generate_checksum(&entity_id_string, ledger_id.as_ref_ledger_id());
        entity_id_string.push('-');
        entity_id_string.push_str(&checksum.0);

        entity_id_string
    }
}

impl Debug for EntityId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "\"{self}\"")
    }
}

impl Display for EntityId {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.shard, self.realm, self.num)
    }
}

impl From<u64> for EntityId {
    fn from(num: u64) -> Self {
        Self { num, shard: 0, realm: 0, checksum: None }
    }
}

impl FromStr for EntityId {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        PartialEntityId::from_str(s)?.finish()
    }
}

#[cfg(test)]
mod tests {
    use crate::ledger_id::RefLedgerId;
    use crate::{
        EntityId,
        TopicId,
    };

    #[test]
    fn from_solidity_address() {
        assert_eq!(
            EntityId::from_solidity_address("000000000000000000000000000000000000138D").unwrap(),
            EntityId { shard: 0, realm: 0, num: 5005, checksum: None }
        );
    }

    #[test]
    fn from_solidity_address_with_0x() {
        assert_eq!(
            EntityId::from_solidity_address("0x000000000000000000000000000000000000138D").unwrap(),
            EntityId { shard: 0, realm: 0, num: 5005, checksum: None }
        );
    }

    #[test]
    fn to_solidity_address() {
        assert!(EntityId { shard: 0, realm: 0, num: 5005, checksum: None }
            .to_solidity_address()
            .unwrap()
            .eq_ignore_ascii_case("000000000000000000000000000000000000138D"));
    }

    #[test]
    fn generate_checksum_mainnet() {
        const EXPECTED: [&str; 256] = [
            "uvnqa", "dfkxr", "lpifi", "tzfmz", "cjcuq", "ktach", "tcxjy", "bmurp", "jwrzg",
            "sgpgx", "hiafh", "rdxmy", "uuuup", "eqscg", "ompjx", "yimro", "iejzf", "sahgw",
            "bweon", "lsbwe", "diuio", "nerqf", "qvoxw", "armfn", "knjne", "ujguv", "efecm",
            "obbkd", "xwyru", "hsvzl", "zjolv", "jfltm", "mwjbd", "wsgiu", "godql", "qkayc",
            "afyft", "kbvnk", "txsvb", "dtqcs", "vkipc", "fgfwt", "ixdek", "stamb", "coxts",
            "mkvbj", "wgsja", "gcpqr", "pymyi", "zukfz", "rlcsj", "tqaaa", "xgxhr", "hcupi",
            "qyrwz", "aupeq", "kqmmh", "umjty", "eihbp", "oeejg", "fuwvq", "pqudh", "thrky",
            "ddosp", "mzmag", "wvjhx", "grgpo", "qndxf", "ajbew", "keymn", "bvqyx", "lrogo",
            "pilof", "zeivw", "jagdn", "swdle", "csasv", "mnyam", "wjvid", "gfspu", "xwlce",
            "hsijv", "ljfrm", "vfczd", "fbagu", "owxol", "ysuwc", "iosdt", "skplk", "cgmtb",
            "txffl", "dtcnc", "hjzut", "rfxck", "bbukb", "kxrrs", "utozj", "epmha", "oljor",
            "yhgwi", "hhghj", "prdpa", "ybawr", "gkyei", "ouvlz", "xestq", "foqbh", "nyniy",
            "wikqp", "eshyg", "euakq", "ndxsh", "vnuzy", "dxshp", "mhppg", "urmwx", "dbkeo",
            "llhmf", "tvetw", "cfcbn", "wbunx", "elrvo", "mvpdf", "vfmkw", "dpjsn", "lzhae",
            "ujehv", "ctbpm", "lcyxd", "tmweu", "toore", "bylyv", "kijgm", "ssgod", "bcdvu",
            "jmbdl", "rvylc", "afvst", "iptak", "qzqib", "rbiul", "zlgcc", "hvdjt", "qfark",
            "yoxzb", "gyvgs", "pisoj", "xspwa", "gcndr", "omkli", "oocxs", "wyafj", "fhxna",
            "nruur", "wbsci", "elpjz", "mvmrq", "vfjzh", "dphgy", "lzeop", "maxaz", "ukuiq",
            "curqh", "leoxy", "tomfp", "byjng", "kigux", "sseco", "bcbkf", "jlyrw", "jnreg",
            "rxolx", "ahlto", "irjbf", "rbgiw", "zldqn", "hvaye", "qeyfv", "yovnm", "gysvd",
            "halhn", "pkipe", "xufwv", "gedem", "ooamd", "wxxtu", "fhvbl", "nrsjc", "wbpqt",
            "elmyk", "enfku", "mxcsl", "vhaac", "dqxht", "maupk", "ukrxb", "cupes", "lemmj",
            "tojua", "byhbr", "klges", "svdmj", "bfaua", "joybr", "ryvji", "aisqz", "ispyq",
            "rcngh", "zmkny", "rthvp", "hyahz", "qhxpq", "yruxh", "hbsey", "plpmp", "xvmug",
            "gfkbx", "ophjo", "wzerf", "pgbyw", "zfulg", "hprsx", "pzpao", "yjmif", "gtjpw",
            "pdgxn", "xnefe", "fxbmv", "ogyum", "gnwcd", "wsoon", "fclwe", "nmjdv", "vwglm",
            "egdtd", "mqbau", "uzyil", "djvqc", "ltsxt", "eaqfk", "ufiru", "cpfzl", "kzdhc",
            "tjaot", "bsxwk", "kcveb", "smsls", "awptj", "jgnba", "bnkir", "rscvb", "acacs",
            "ilxkj", "qvusa", "zfrzr", "hpphi",
        ];

        for (index, expected) in EXPECTED.iter().enumerate() {
            let actual = EntityId::generate_checksum(
                &TopicId::from(index as u64).to_string(),
                &RefLedgerId::MAINNET,
            )
            .to_string();

            assert_eq!(expected, &actual);
        }
    }

    #[test]
    fn generate_checksum_testnet() {
        const EXPECTED: [&str; 256] = [
            "eiyxj", "mswfa", "vctmr", "dmqui", "lwobz", "ugljq", "cqirh", "lafyy", "tkdgp",
            "buaog", "qvlmq", "ariuh", "eigby", "oedjp", "yaarg", "hvxyx", "rrvgo", "bnsof",
            "ljpvw", "vfndn", "mwfpx", "wscxo", "ajaff", "kexmw", "uauun", "dwsce", "nspjv",
            "xomrm", "hkjzd", "rghgu", "iwzte", "ssxav", "wjuim", "gfrqd", "qboxu", "zxmfl",
            "jtjnc", "tpgut", "dleck", "nhbkb", "extwl", "otrec", "skolt", "cgltk", "mcjbb",
            "vygis", "fudqj", "pqaya", "zlyfr", "jhvni", "aynzs", "ddlhj", "guipa", "qqfwr",
            "amdei", "kialz", "udxtq", "dzvbh", "nvsiy", "xrpqp", "piicz", "zefkq", "cvcsh",
            "mqzzy", "wmxhp", "giupg", "qerwx", "aapeo", "jwmmf", "tsjtw", "ljcgg", "veznx",
            "yvwvo", "irudf", "snrkw", "cjosn", "mfmae", "wbjhv", "fxgpm", "ptdxd", "hjwjn",
            "rftre", "uwqyv", "esogm", "oolod", "ykivu", "iggdl", "scdlc", "byast", "ltyak",
            "dkqmu", "ngnul", "qxlcc", "atijt", "kpfrk", "ulczb", "ehags", "ocxoj", "xyuwa",
            "husdr", "quros", "zeowj", "homea", "pyjlr", "yigti", "gseaz", "pcbiq", "xlyqh",
            "fvvxy", "oftfp", "ohlrz", "wrizq", "fbghh", "nldoy", "vvawp", "eeyeg", "movlx",
            "uysto", "diqbf", "lsniw", "fpfvg", "nzdcx", "wjako", "esxsf", "ncuzw", "vmshn",
            "dwppe", "mgmwv", "uqkem", "dahmd", "dbzyn", "llxge", "tvunv", "cfrvm", "kppdd",
            "szmku", "bjjsl", "jthac", "sdeht", "anbpk", "aoubu", "iyrjl", "riorc", "zslyt",
            "icjgk", "qmgob", "ywdvs", "hgbdj", "ppyla", "xzvsr", "ybofb", "gllms", "oviuj",
            "xfgca", "fpdjr", "nzari", "wixyz", "esvgq", "ncsoh", "vmpvy", "voiii", "dyfpz",
            "micxq", "usafh", "dbxmy", "lluup", "tvscg", "cfpjx", "kpmro", "szjzf", "tbclp",
            "bkztg", "juxax", "seuio", "aorqf", "iyoxw", "rimfn", "zsjne", "icguv", "qmecm",
            "qnwow", "yxtwn", "hhree", "prolv", "ybltm", "gljbd", "ovgiu", "xfdql", "fpayc",
            "nyyft", "oaqsd", "wknzu", "eulhl", "neipc", "vofwt", "dydek", "miamb", "urxts",
            "dbvbj", "llsja", "tyrmb", "ciots", "ksmbj", "tcjja", "bmgqr", "jwdyi", "sgbfz",
            "apynq", "izvvh", "bgtcy", "rllpi", "zviwz", "ifgeq", "qpdmh", "yzaty", "hiybp",
            "psvjg", "ycsqx", "gmpyo", "ytngf", "itfsp", "rddag", "znahx", "hwxpo", "qguxf",
            "yqsew", "hapmn", "pkmue", "xukbv", "qbhjm", "gfzvw", "opxdn", "wzule", "fjrsv",
            "ntpam", "wdmid", "enjpu", "mxgxl", "vhefc", "nobmt", "dstzd", "mcrgu", "umool",
            "cwlwc", "lgjdt", "tqglk", "cadtb", "kkbas", "styij", "lavqa", "bfock", "jplkb",
            "rzirs", "ajfzj", "itdha", "rdaor",
        ];

        for (index, expected) in EXPECTED.iter().enumerate() {
            let actual = EntityId::generate_checksum(
                &TopicId::from(index as u64).to_string(),
                RefLedgerId::TESTNET,
            )
            .to_string();

            assert_eq!(expected, &actual);
        }
    }
}
