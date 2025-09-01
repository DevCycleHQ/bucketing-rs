mod constants {}
pub const OPERATOR_AND: &str = "and";
pub const OPERATOR_OR: &str = "or";

pub const VARIABLE_EVALUATED_EVENT: &str = "variableEvaluated";
pub const VARIABLE_DEFAULTED_EVENT: &str = "variableDefaulted";
pub const AGG_VARIABLE_EVALUATED_EVENT: &str = "aggVariableEvaluated";
pub const AGG_VARIABLE_DEFAULTED_EVENT: &str = "aggVariableDefaulted";

pub const TYPE_ALL: &str = "all";
pub const TYPE_USER: &str = "user";
pub const TYPE_OPT_IN: &str = "optIn";
pub const TYPE_AUDIENCE_MATCH: &str = "audienceMatch";

pub const SUB_TYPE_USER_ID: &str = "user_id";
pub const SUB_TYPE_EMAIL: &str = "email";
pub const SUB_TYPE_IP: &str = "ip";
pub const SUB_TYPE_COUNTRY: &str = "country";
pub const SUB_TYPE_PLATFORM: &str = "platform";
pub const SUB_TYPE_PLATFORM_VERSION: &str = "platformVersion";
pub const SUB_TYPE_APP_VERSION: &str = "appVersion";
pub const SUB_TYPE_DEVICE_MODEL: &str = "deviceModel";
pub const SUB_TYPE_CUSTOM_DATA: &str = "customData";

pub const COMPARATOR_EQUAL: &str = "=";
pub const COMPARATOR_NOT_EQUAL: &str = "!=";
pub const COMPARATOR_GREATER: &str = ">";
pub const COMPARATOR_GREATER_EQUAL: &str = ">=";
pub const COMPARATOR_LESS: &str = "<";
pub const COMPARATOR_LESS_EQUAL: &str = "<=";
pub const COMPARATOR_EXIST: &str = "exist";
pub const COMPARATOR_NOT_EXIST: &str = "!exist";
pub const COMPARATOR_CONTAIN: &str = "contain";
pub const COMPARATOR_NOT_CONTAIN: &str = "!contain";
pub const DATA_KEY_TYPE_STRING: &str = "String";
pub const DATA_KEY_TYPE_BOOLEAN: &str = "Boolean";
pub const DATA_KEY_TYPE_NUMBER: &str = "Number";

pub const VARIABLE_TYPES_STRING: &str = "String";
pub const VARIABLE_TYPES_NUMBER: &str = "Number";
pub const VARIABLE_TYPES_JSON: &str = "JSON";
pub const VARIABLE_TYPES_BOOL: &str = "Boolean";

pub const ROLLOUT_TYPE_SCHEDULE: &str = "schedule";
pub const ROLLOUT_TYPE_PERCENTAGE: &str = "percentage";
pub const ROLLOUT_TYPE_DISCRETE: &str = "discrete";

pub const DEFAULT_BUCKETING_VALUE: &str = "null";

pub const BASE_SEED: u32 = 1;
pub const MAX_HASH_VALUE: u32 = 4294967295;
