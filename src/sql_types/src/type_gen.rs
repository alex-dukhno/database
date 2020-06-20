// Autogenerated file - DO NOT EDIT

#[derive(PartialEq, Eq, Clone, Debug, Hash)]
pub enum SqlType {
    /// BOOL - boolean, &#39;true&#39;/&#39;false&#39;
    Bool,
    /// BYTEA - variable-length string, binary values escaped
    Bytea,
    /// CHAR - single character
    Char,
    /// NAME - 63-byte type for storing system identifiers
    Name,
    /// INT8 - ~18 digit integer, 8-byte storage
    Int8,
    /// INT2 - -32 thousand to 32 thousand, 2-byte storage
    Int2,
    /// INT2VECTOR - array of int2, used in system tables
    Int2Vector,
    /// INT4 - -2 billion to 2 billion integer, 4-byte storage
    Int4,
    /// REGPROC - registered procedure
    Regproc,
    /// TEXT - variable-length string, no limit specified
    Text,
    /// OID - object identifier&#40;oid&#41;, maximum 4 billion
    Oid,
    /// TID - &#40;block, offset&#41;, physical location of tuple
    Tid,
    /// XID - transaction id
    Xid,
    /// CID - command identifier type, sequence in transaction id
    Cid,
    /// OIDVECTOR - array of oids, used in system tables
    OidVector,
    /// PG_DDL_COMMAND - internal type for passing CollectedCommand
    PgDdlCommand,
    /// PG_TYPE
    PgType,
    /// PG_ATTRIBUTE
    PgAttribute,
    /// PG_PROC
    PgProc,
    /// PG_CLASS
    PgClass,
    /// JSON - JSON stored as text
    Json,
    /// XML - XML content
    Xml,
    /// PG_NODE_TREE - string representing an internal node tree
    PgNodeTree,
    /// TABLE_AM_HANDLER
    TableAmHandler,
    /// INDEX_AM_HANDLER - pseudo-type for the result of an index AM handler function
    IndexAmHandler,
    /// POINT - geometric point &#39;&#40;x, y&#41;&#39;
    Point,
    /// LSEG - geometric line segment &#39;&#40;pt1,pt2&#41;&#39;
    Lseg,
    /// PATH - geometric path &#39;&#40;pt1,...&#41;&#39;
    Path,
    /// BOX - geometric box &#39;&#40;lower left,upper right&#41;&#39;
    Box,
    /// POLYGON - geometric polygon &#39;&#40;pt1,...&#41;&#39;
    Polygon,
    /// LINE - geometric line
    Line,
    /// CIDR - network IP address/netmask, network address
    Cidr,
    /// FLOAT4 - single-precision floating point number, 4-byte storage
    Float4,
    /// FLOAT8 - double-precision floating point number, 8-byte storage
    Float8,
    /// UNKNOWN - pseudo-type representing an undetermined type
    Unknown,
    /// CIRCLE - geometric circle &#39;&#40;center,radius&#41;&#39;
    Circle,
    /// MACADDR8 - XX:XX:XX:XX:XX:XX:XX:XX, MAC address
    Macaddr8,
    /// MONEY - monetary amounts, &#36;d,ddd.cc
    Money,
    /// MACADDR - XX:XX:XX:XX:XX:XX, MAC address
    Macaddr,
    /// INET - IP address/netmask, host address, netmask optional
    Inet,
    /// ACLITEM - access control list
    Aclitem,
    /// BPCHAR - char&#40;length&#41;, blank-padded string, fixed storage length
    Bpchar,
    /// VARCHAR - varchar&#40;length&#41;, non-blank-padded string, variable storage length
    Varchar,
    /// DATE - date
    Date,
    /// TIME - time of day
    Time,
    /// TIMESTAMP - date and time
    Timestamp,
    /// TIMESTAMPTZ - date and time with time zone
    Timestamptz,
    /// INTERVAL - &#64; &lt;number&gt; &lt;units&gt;, time interval
    Interval,
    /// TIMETZ - time of day with time zone
    Timetz,
    /// BIT - fixed-length bit string
    Bit,
    /// VARBIT - variable-length bit string
    Varbit,
    /// NUMERIC - numeric&#40;precision, decimal&#41;, arbitrary precision number
    Numeric,
    /// REFCURSOR - reference to cursor &#40;portal name&#41;
    Refcursor,
    /// REGPROCEDURE - registered procedure &#40;with args&#41;
    Regprocedure,
    /// REGOPER - registered operator
    Regoper,
    /// REGOPERATOR - registered operator &#40;with args&#41;
    Regoperator,
    /// REGCLASS - registered class
    Regclass,
    /// REGTYPE - registered type
    Regtype,
    /// RECORD - pseudo-type representing any composite type
    Record,
    /// CSTRING - C-style string
    Cstring,
    /// ANY - pseudo-type representing any type
    Any,
    /// ANYARRAY - pseudo-type representing a polymorphic array type
    Anyarray,
    /// VOID - pseudo-type for the result of a function with no real result
    Void,
    /// TRIGGER - pseudo-type for the result of a trigger function
    Trigger,
    /// LANGUAGE_HANDLER - pseudo-type for the result of a language handler function
    LanguageHandler,
    /// INTERNAL - pseudo-type representing an internal data structure
    Internal,
    /// OPAQUE - obsolete, deprecated pseudo-type
    Opaque,
    /// ANYELEMENT - pseudo-type representing a polymorphic base type
    Anyelement,
    /// RECORD&#91;&#93;
    RecordArray,
    /// ANYNONARRAY - pseudo-type representing a polymorphic base type that is not an array
    Anynonarray,
    /// UUID - UUID datatype
    Uuid,
    /// TXID_SNAPSHOT - txid snapshot
    TxidSnapshot,
    /// FDW_HANDLER - pseudo-type for the result of an FDW handler function
    FdwHandler,
    /// PG_LSN - PostgreSQL LSN datatype
    PgLsn,
    /// TSM_HANDLER - pseudo-type for the result of a tablesample method function
    TsmHandler,
    /// PG_NDISTINCT - multivariate ndistinct coefficients
    PgNdistinct,
    /// PG_DEPENDENCIES - multivariate dependencies
    PgDependencies,
    /// ANYENUM - pseudo-type representing a polymorphic base type that is an enum
    Anyenum,
    /// TSVECTOR - text representation for text search
    TsVector,
    /// TSQUERY - query representation for text search
    Tsquery,
    /// GTSVECTOR - GiST index internal text representation for text search
    GtsVector,
    /// REGCONFIG - registered text search configuration
    Regconfig,
    /// REGDICTIONARY - registered text search dictionary
    Regdictionary,
    /// JSONB - Binary JSON
    Jsonb,
    /// ANYRANGE - pseudo-type representing a polymorphic base type that is a range
    AnyRange,
    /// EVENT_TRIGGER - pseudo-type for the result of an event trigger function
    EventTrigger,
    /// INT4RANGE - range of integers
    Int4Range,
    /// NUMRANGE - range of numerics
    NumRange,
    /// TSRANGE - range of timestamps without time zone
    TsRange,
    /// TSTZRANGE - range of timestamps with time zone
    TstzRange,
    /// DATERANGE - range of dates
    DateRange,
    /// INT8RANGE - range of bigints
    Int8Range,
    /// JSONPATH - JSON path
    Jsonpath,
    /// REGNAMESPACE - registered namespace
    Regnamespace,
    /// REGROLE - registered role
    Regrole,
    /// PG_MCV_LIST - multivariate MCV list
    PgMcvList,
}

#[allow(clippy::len_without_is_empty)]
impl SqlType {
    pub fn from_oid(oid: i32) -> Option<SqlType> {
        match oid {
            16 => Some(SqlType::Bool),
            17 => Some(SqlType::Bytea),
            18 => Some(SqlType::Char),
            19 => Some(SqlType::Name),
            20 => Some(SqlType::Int8),
            21 => Some(SqlType::Int2),
            22 => Some(SqlType::Int2Vector),
            23 => Some(SqlType::Int4),
            24 => Some(SqlType::Regproc),
            25 => Some(SqlType::Text),
            26 => Some(SqlType::Oid),
            27 => Some(SqlType::Tid),
            28 => Some(SqlType::Xid),
            29 => Some(SqlType::Cid),
            30 => Some(SqlType::OidVector),
            32 => Some(SqlType::PgDdlCommand),
            71 => Some(SqlType::PgType),
            75 => Some(SqlType::PgAttribute),
            81 => Some(SqlType::PgProc),
            83 => Some(SqlType::PgClass),
            114 => Some(SqlType::Json),
            142 => Some(SqlType::Xml),
            194 => Some(SqlType::PgNodeTree),
            269 => Some(SqlType::TableAmHandler),
            325 => Some(SqlType::IndexAmHandler),
            600 => Some(SqlType::Point),
            601 => Some(SqlType::Lseg),
            602 => Some(SqlType::Path),
            603 => Some(SqlType::Box),
            604 => Some(SqlType::Polygon),
            628 => Some(SqlType::Line),
            650 => Some(SqlType::Cidr),
            700 => Some(SqlType::Float4),
            701 => Some(SqlType::Float8),
            705 => Some(SqlType::Unknown),
            718 => Some(SqlType::Circle),
            774 => Some(SqlType::Macaddr8),
            790 => Some(SqlType::Money),
            829 => Some(SqlType::Macaddr),
            869 => Some(SqlType::Inet),
            1033 => Some(SqlType::Aclitem),
            1042 => Some(SqlType::Bpchar),
            1043 => Some(SqlType::Varchar),
            1082 => Some(SqlType::Date),
            1083 => Some(SqlType::Time),
            1114 => Some(SqlType::Timestamp),
            1184 => Some(SqlType::Timestamptz),
            1186 => Some(SqlType::Interval),
            1266 => Some(SqlType::Timetz),
            1560 => Some(SqlType::Bit),
            1562 => Some(SqlType::Varbit),
            1700 => Some(SqlType::Numeric),
            1790 => Some(SqlType::Refcursor),
            2202 => Some(SqlType::Regprocedure),
            2203 => Some(SqlType::Regoper),
            2204 => Some(SqlType::Regoperator),
            2205 => Some(SqlType::Regclass),
            2206 => Some(SqlType::Regtype),
            2249 => Some(SqlType::Record),
            2275 => Some(SqlType::Cstring),
            2276 => Some(SqlType::Any),
            2277 => Some(SqlType::Anyarray),
            2278 => Some(SqlType::Void),
            2279 => Some(SqlType::Trigger),
            2280 => Some(SqlType::LanguageHandler),
            2281 => Some(SqlType::Internal),
            2282 => Some(SqlType::Opaque),
            2283 => Some(SqlType::Anyelement),
            2287 => Some(SqlType::RecordArray),
            2776 => Some(SqlType::Anynonarray),
            2950 => Some(SqlType::Uuid),
            2970 => Some(SqlType::TxidSnapshot),
            3115 => Some(SqlType::FdwHandler),
            3220 => Some(SqlType::PgLsn),
            3310 => Some(SqlType::TsmHandler),
            3361 => Some(SqlType::PgNdistinct),
            3402 => Some(SqlType::PgDependencies),
            3500 => Some(SqlType::Anyenum),
            3614 => Some(SqlType::TsVector),
            3615 => Some(SqlType::Tsquery),
            3642 => Some(SqlType::GtsVector),
            3734 => Some(SqlType::Regconfig),
            3769 => Some(SqlType::Regdictionary),
            3802 => Some(SqlType::Jsonb),
            3831 => Some(SqlType::AnyRange),
            3838 => Some(SqlType::EventTrigger),
            3904 => Some(SqlType::Int4Range),
            3906 => Some(SqlType::NumRange),
            3908 => Some(SqlType::TsRange),
            3910 => Some(SqlType::TstzRange),
            3912 => Some(SqlType::DateRange),
            3926 => Some(SqlType::Int8Range),
            4072 => Some(SqlType::Jsonpath),
            4089 => Some(SqlType::Regnamespace),
            4096 => Some(SqlType::Regrole),
            5017 => Some(SqlType::PgMcvList),
            _ => None,
        }
    }

    pub fn oid(&self) -> i32 {
        match *self {
            SqlType::Bool => 16,
            SqlType::Bytea => 17,
            SqlType::Char => 18,
            SqlType::Name => 19,
            SqlType::Int8 => 20,
            SqlType::Int2 => 21,
            SqlType::Int2Vector => 22,
            SqlType::Int4 => 23,
            SqlType::Regproc => 24,
            SqlType::Text => 25,
            SqlType::Oid => 26,
            SqlType::Tid => 27,
            SqlType::Xid => 28,
            SqlType::Cid => 29,
            SqlType::OidVector => 30,
            SqlType::PgDdlCommand => 32,
            SqlType::PgType => 71,
            SqlType::PgAttribute => 75,
            SqlType::PgProc => 81,
            SqlType::PgClass => 83,
            SqlType::Json => 114,
            SqlType::Xml => 142,
            SqlType::PgNodeTree => 194,
            SqlType::TableAmHandler => 269,
            SqlType::IndexAmHandler => 325,
            SqlType::Point => 600,
            SqlType::Lseg => 601,
            SqlType::Path => 602,
            SqlType::Box => 603,
            SqlType::Polygon => 604,
            SqlType::Line => 628,
            SqlType::Cidr => 650,
            SqlType::Float4 => 700,
            SqlType::Float8 => 701,
            SqlType::Unknown => 705,
            SqlType::Circle => 718,
            SqlType::Macaddr8 => 774,
            SqlType::Money => 790,
            SqlType::Macaddr => 829,
            SqlType::Inet => 869,
            SqlType::Aclitem => 1033,
            SqlType::Bpchar => 1042,
            SqlType::Varchar => 1043,
            SqlType::Date => 1082,
            SqlType::Time => 1083,
            SqlType::Timestamp => 1114,
            SqlType::Timestamptz => 1184,
            SqlType::Interval => 1186,
            SqlType::Timetz => 1266,
            SqlType::Bit => 1560,
            SqlType::Varbit => 1562,
            SqlType::Numeric => 1700,
            SqlType::Refcursor => 1790,
            SqlType::Regprocedure => 2202,
            SqlType::Regoper => 2203,
            SqlType::Regoperator => 2204,
            SqlType::Regclass => 2205,
            SqlType::Regtype => 2206,
            SqlType::Record => 2249,
            SqlType::Cstring => 2275,
            SqlType::Any => 2276,
            SqlType::Anyarray => 2277,
            SqlType::Void => 2278,
            SqlType::Trigger => 2279,
            SqlType::LanguageHandler => 2280,
            SqlType::Internal => 2281,
            SqlType::Opaque => 2282,
            SqlType::Anyelement => 2283,
            SqlType::RecordArray => 2287,
            SqlType::Anynonarray => 2776,
            SqlType::Uuid => 2950,
            SqlType::TxidSnapshot => 2970,
            SqlType::FdwHandler => 3115,
            SqlType::PgLsn => 3220,
            SqlType::TsmHandler => 3310,
            SqlType::PgNdistinct => 3361,
            SqlType::PgDependencies => 3402,
            SqlType::Anyenum => 3500,
            SqlType::TsVector => 3614,
            SqlType::Tsquery => 3615,
            SqlType::GtsVector => 3642,
            SqlType::Regconfig => 3734,
            SqlType::Regdictionary => 3769,
            SqlType::Jsonb => 3802,
            SqlType::AnyRange => 3831,
            SqlType::EventTrigger => 3838,
            SqlType::Int4Range => 3904,
            SqlType::NumRange => 3906,
            SqlType::TsRange => 3908,
            SqlType::TstzRange => 3910,
            SqlType::DateRange => 3912,
            SqlType::Int8Range => 3926,
            SqlType::Jsonpath => 4072,
            SqlType::Regnamespace => 4089,
            SqlType::Regrole => 4096,
            SqlType::PgMcvList => 5017,
        }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<SqlType> {
        match s {
            "bool" => Some(SqlType::Bool),
            "bytea" => Some(SqlType::Bytea),
            "char" => Some(SqlType::Char),
            "name" => Some(SqlType::Name),
            "int8" => Some(SqlType::Int8),
            "int2" => Some(SqlType::Int2),
            "int2vector" => Some(SqlType::Int2Vector),
            "int4" => Some(SqlType::Int4),
            "regproc" => Some(SqlType::Regproc),
            "text" => Some(SqlType::Text),
            "oid" => Some(SqlType::Oid),
            "tid" => Some(SqlType::Tid),
            "xid" => Some(SqlType::Xid),
            "cid" => Some(SqlType::Cid),
            "oidvector" => Some(SqlType::OidVector),
            "pg_ddl_command" => Some(SqlType::PgDdlCommand),
            "pg_type" => Some(SqlType::PgType),
            "pg_attribute" => Some(SqlType::PgAttribute),
            "pg_proc" => Some(SqlType::PgProc),
            "pg_class" => Some(SqlType::PgClass),
            "json" => Some(SqlType::Json),
            "xml" => Some(SqlType::Xml),
            "pg_node_tree" => Some(SqlType::PgNodeTree),
            "table_am_handler" => Some(SqlType::TableAmHandler),
            "index_am_handler" => Some(SqlType::IndexAmHandler),
            "point" => Some(SqlType::Point),
            "lseg" => Some(SqlType::Lseg),
            "path" => Some(SqlType::Path),
            "box" => Some(SqlType::Box),
            "polygon" => Some(SqlType::Polygon),
            "line" => Some(SqlType::Line),
            "cidr" => Some(SqlType::Cidr),
            "float4" => Some(SqlType::Float4),
            "float8" => Some(SqlType::Float8),
            "unknown" => Some(SqlType::Unknown),
            "circle" => Some(SqlType::Circle),
            "macaddr8" => Some(SqlType::Macaddr8),
            "money" => Some(SqlType::Money),
            "macaddr" => Some(SqlType::Macaddr),
            "inet" => Some(SqlType::Inet),
            "aclitem" => Some(SqlType::Aclitem),
            "bpchar" => Some(SqlType::Bpchar),
            "varchar" => Some(SqlType::Varchar),
            "date" => Some(SqlType::Date),
            "time" => Some(SqlType::Time),
            "timestamp" => Some(SqlType::Timestamp),
            "timestamptz" => Some(SqlType::Timestamptz),
            "interval" => Some(SqlType::Interval),
            "timetz" => Some(SqlType::Timetz),
            "bit" => Some(SqlType::Bit),
            "varbit" => Some(SqlType::Varbit),
            "numeric" => Some(SqlType::Numeric),
            "refcursor" => Some(SqlType::Refcursor),
            "regprocedure" => Some(SqlType::Regprocedure),
            "regoper" => Some(SqlType::Regoper),
            "regoperator" => Some(SqlType::Regoperator),
            "regclass" => Some(SqlType::Regclass),
            "regtype" => Some(SqlType::Regtype),
            "record" => Some(SqlType::Record),
            "cstring" => Some(SqlType::Cstring),
            "any" => Some(SqlType::Any),
            "anyarray" => Some(SqlType::Anyarray),
            "void" => Some(SqlType::Void),
            "trigger" => Some(SqlType::Trigger),
            "language_handler" => Some(SqlType::LanguageHandler),
            "internal" => Some(SqlType::Internal),
            "opaque" => Some(SqlType::Opaque),
            "anyelement" => Some(SqlType::Anyelement),
            "_record" => Some(SqlType::RecordArray),
            "anynonarray" => Some(SqlType::Anynonarray),
            "uuid" => Some(SqlType::Uuid),
            "txid_snapshot" => Some(SqlType::TxidSnapshot),
            "fdw_handler" => Some(SqlType::FdwHandler),
            "pg_lsn" => Some(SqlType::PgLsn),
            "tsm_handler" => Some(SqlType::TsmHandler),
            "pg_ndistinct" => Some(SqlType::PgNdistinct),
            "pg_dependencies" => Some(SqlType::PgDependencies),
            "anyenum" => Some(SqlType::Anyenum),
            "tsvector" => Some(SqlType::TsVector),
            "tsquery" => Some(SqlType::Tsquery),
            "gtsvector" => Some(SqlType::GtsVector),
            "regconfig" => Some(SqlType::Regconfig),
            "regdictionary" => Some(SqlType::Regdictionary),
            "jsonb" => Some(SqlType::Jsonb),
            "anyrange" => Some(SqlType::AnyRange),
            "event_trigger" => Some(SqlType::EventTrigger),
            "int4range" => Some(SqlType::Int4Range),
            "numrange" => Some(SqlType::NumRange),
            "tsrange" => Some(SqlType::TsRange),
            "tstzrange" => Some(SqlType::TstzRange),
            "daterange" => Some(SqlType::DateRange),
            "int8range" => Some(SqlType::Int8Range),
            "jsonpath" => Some(SqlType::Jsonpath),
            "regnamespace" => Some(SqlType::Regnamespace),
            "regrole" => Some(SqlType::Regrole),
            "pg_mcv_list" => Some(SqlType::PgMcvList),
            _ => None,
        }
    }

    pub fn name(&self) -> &str {
        match *self {
            SqlType::Bool => "bool",
            SqlType::Bytea => "bytea",
            SqlType::Char => "char",
            SqlType::Name => "name",
            SqlType::Int8 => "int8",
            SqlType::Int2 => "int2",
            SqlType::Int2Vector => "int2vector",
            SqlType::Int4 => "int4",
            SqlType::Regproc => "regproc",
            SqlType::Text => "text",
            SqlType::Oid => "oid",
            SqlType::Tid => "tid",
            SqlType::Xid => "xid",
            SqlType::Cid => "cid",
            SqlType::OidVector => "oidvector",
            SqlType::PgDdlCommand => "pg_ddl_command",
            SqlType::PgType => "pg_type",
            SqlType::PgAttribute => "pg_attribute",
            SqlType::PgProc => "pg_proc",
            SqlType::PgClass => "pg_class",
            SqlType::Json => "json",
            SqlType::Xml => "xml",
            SqlType::PgNodeTree => "pg_node_tree",
            SqlType::TableAmHandler => "table_am_handler",
            SqlType::IndexAmHandler => "index_am_handler",
            SqlType::Point => "point",
            SqlType::Lseg => "lseg",
            SqlType::Path => "path",
            SqlType::Box => "box",
            SqlType::Polygon => "polygon",
            SqlType::Line => "line",
            SqlType::Cidr => "cidr",
            SqlType::Float4 => "float4",
            SqlType::Float8 => "float8",
            SqlType::Unknown => "unknown",
            SqlType::Circle => "circle",
            SqlType::Macaddr8 => "macaddr8",
            SqlType::Money => "money",
            SqlType::Macaddr => "macaddr",
            SqlType::Inet => "inet",
            SqlType::Aclitem => "aclitem",
            SqlType::Bpchar => "bpchar",
            SqlType::Varchar => "varchar",
            SqlType::Date => "date",
            SqlType::Time => "time",
            SqlType::Timestamp => "timestamp",
            SqlType::Timestamptz => "timestamptz",
            SqlType::Interval => "interval",
            SqlType::Timetz => "timetz",
            SqlType::Bit => "bit",
            SqlType::Varbit => "varbit",
            SqlType::Numeric => "numeric",
            SqlType::Refcursor => "refcursor",
            SqlType::Regprocedure => "regprocedure",
            SqlType::Regoper => "regoper",
            SqlType::Regoperator => "regoperator",
            SqlType::Regclass => "regclass",
            SqlType::Regtype => "regtype",
            SqlType::Record => "record",
            SqlType::Cstring => "cstring",
            SqlType::Any => "any",
            SqlType::Anyarray => "anyarray",
            SqlType::Void => "void",
            SqlType::Trigger => "trigger",
            SqlType::LanguageHandler => "language_handler",
            SqlType::Internal => "internal",
            SqlType::Opaque => "opaque",
            SqlType::Anyelement => "anyelement",
            SqlType::RecordArray => "_record",
            SqlType::Anynonarray => "anynonarray",
            SqlType::Uuid => "uuid",
            SqlType::TxidSnapshot => "txid_snapshot",
            SqlType::FdwHandler => "fdw_handler",
            SqlType::PgLsn => "pg_lsn",
            SqlType::TsmHandler => "tsm_handler",
            SqlType::PgNdistinct => "pg_ndistinct",
            SqlType::PgDependencies => "pg_dependencies",
            SqlType::Anyenum => "anyenum",
            SqlType::TsVector => "tsvector",
            SqlType::Tsquery => "tsquery",
            SqlType::GtsVector => "gtsvector",
            SqlType::Regconfig => "regconfig",
            SqlType::Regdictionary => "regdictionary",
            SqlType::Jsonb => "jsonb",
            SqlType::AnyRange => "anyrange",
            SqlType::EventTrigger => "event_trigger",
            SqlType::Int4Range => "int4range",
            SqlType::NumRange => "numrange",
            SqlType::TsRange => "tsrange",
            SqlType::TstzRange => "tstzrange",
            SqlType::DateRange => "daterange",
            SqlType::Int8Range => "int8range",
            SqlType::Jsonpath => "jsonpath",
            SqlType::Regnamespace => "regnamespace",
            SqlType::Regrole => "regrole",
            SqlType::PgMcvList => "pg_mcv_list",
        }
    }

    pub fn len(&self) -> i16 {
        match *self {
            SqlType::Bool => 1,
            SqlType::Bytea => -1,
            SqlType::Char => 1,
            SqlType::Name => 0,
            SqlType::Int8 => 8,
            SqlType::Int2 => 2,
            SqlType::Int2Vector => -1,
            SqlType::Int4 => 4,
            SqlType::Regproc => 4,
            SqlType::Text => -1,
            SqlType::Oid => 4,
            SqlType::Tid => 6,
            SqlType::Xid => 4,
            SqlType::Cid => 4,
            SqlType::OidVector => -1,
            SqlType::PgDdlCommand => 0,
            SqlType::PgType => -1,
            SqlType::PgAttribute => -1,
            SqlType::PgProc => -1,
            SqlType::PgClass => -1,
            SqlType::Json => -1,
            SqlType::Xml => -1,
            SqlType::PgNodeTree => -1,
            SqlType::TableAmHandler => 4,
            SqlType::IndexAmHandler => 4,
            SqlType::Point => 16,
            SqlType::Lseg => 32,
            SqlType::Path => -1,
            SqlType::Box => 32,
            SqlType::Polygon => -1,
            SqlType::Line => 24,
            SqlType::Cidr => -1,
            SqlType::Float4 => 4,
            SqlType::Float8 => 8,
            SqlType::Unknown => -2,
            SqlType::Circle => 24,
            SqlType::Macaddr8 => 8,
            SqlType::Money => 8,
            SqlType::Macaddr => 6,
            SqlType::Inet => -1,
            SqlType::Aclitem => 12,
            SqlType::Bpchar => -1,
            SqlType::Varchar => -1,
            SqlType::Date => 4,
            SqlType::Time => 8,
            SqlType::Timestamp => 8,
            SqlType::Timestamptz => 8,
            SqlType::Interval => 16,
            SqlType::Timetz => 12,
            SqlType::Bit => -1,
            SqlType::Varbit => -1,
            SqlType::Numeric => -1,
            SqlType::Refcursor => -1,
            SqlType::Regprocedure => 4,
            SqlType::Regoper => 4,
            SqlType::Regoperator => 4,
            SqlType::Regclass => 4,
            SqlType::Regtype => 4,
            SqlType::Record => -1,
            SqlType::Cstring => -2,
            SqlType::Any => 4,
            SqlType::Anyarray => -1,
            SqlType::Void => 4,
            SqlType::Trigger => 4,
            SqlType::LanguageHandler => 4,
            SqlType::Internal => 0,
            SqlType::Opaque => 4,
            SqlType::Anyelement => 4,
            SqlType::RecordArray => -1,
            SqlType::Anynonarray => 4,
            SqlType::Uuid => 16,
            SqlType::TxidSnapshot => -1,
            SqlType::FdwHandler => 4,
            SqlType::PgLsn => 8,
            SqlType::TsmHandler => 4,
            SqlType::PgNdistinct => -1,
            SqlType::PgDependencies => -1,
            SqlType::Anyenum => 4,
            SqlType::TsVector => -1,
            SqlType::Tsquery => -1,
            SqlType::GtsVector => -1,
            SqlType::Regconfig => 4,
            SqlType::Regdictionary => 4,
            SqlType::Jsonb => -1,
            SqlType::AnyRange => -1,
            SqlType::EventTrigger => 4,
            SqlType::Int4Range => -1,
            SqlType::NumRange => -1,
            SqlType::TsRange => -1,
            SqlType::TstzRange => -1,
            SqlType::DateRange => -1,
            SqlType::Int8Range => -1,
            SqlType::Jsonpath => -1,
            SqlType::Regnamespace => 4,
            SqlType::Regrole => 4,
            SqlType::PgMcvList => -1,
        }
    }
}
