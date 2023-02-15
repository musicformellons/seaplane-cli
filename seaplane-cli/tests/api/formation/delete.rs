//! Note we only test `formation delete` combinations that touch the API
use httpmock::prelude::*;
use seaplane::api::compute::v2::{PageMetadata, PagedResponse};
use seaplane_cli::{printer::printer, test_main_exec_with_ctx};

use crate::api::{
    formation::{
        ctx_with_formations, default_deployed_formation, default_local_formation,
        multi_deployed_formations,
    },
    when, MOCK_SERVER,
};

macro_rules! test_fn_delete {
    (@impl $test_fn:ident, $argv:expr, $ctx:expr, $err:expr, $after:expr) => {
        #[test]
        fn $test_fn() {
            let (hit, correct_out) = if $argv.contains("--no-remote") {
                (false, "Successfully removed 1 item from the local DB".to_string())
            } else {
                (
                    true,
                    format!("Successfully Landed remote Formation Instance \
                    frm-bcbdixdcojdu3o67lbh2gflaxe{}", $after)
                )
            };

            mock_land!($argv, $ctx, correct_out, $err, hit);
            printer().clear();
        }
    };
    (@is_err $test_fn:ident, $argv:expr, $local_db:expr) => {
        test_fn_delete!(@impl
                        $test_fn,
                        $argv,
                        ctx_with_formations(vec![$local_db]),
                        true,
                        " (stubb)\n\nSuccessfully removed 1 item from the local DB");
    };
    (@no_name $test_fn:ident, $argv:expr, $local_db:expr) => {
        test_fn_delete!(@impl
                        $test_fn,
                        $argv,
                        ctx_with_formations(vec![$local_db]),
                        false,
                        "");
    };
    ($test_fn:ident, $argv:expr, $local_db:expr) => {
        test_fn_delete!(@impl
                        $test_fn,
                        $argv,
                        ctx_with_formations(vec![$local_db]),
                        false,
                        " (stubb)\n\nSuccessfully removed 1 item from the local DB");
    };
}

// Delete a single formation by name where we already know the OID
test_fn_delete!(name_has_local_oid, "formation delete stubb", default_deployed_formation());
// If we don't already have OIDs populated we should get a CLI error saying we dont know the OID of
// name "stubb"
test_fn_delete!(
    @is_err
    name_no_local_oid,
    "formation delete stubb",
    default_local_formation()
);
// --fetch fixes that
test_fn_delete!(
    name_no_local_oid_fetch,
    "formation delete stubb --fetch",
    default_local_formation()
);
// using --fetch and already knowing the OID should be no problem
test_fn_delete!(
    name_has_local_oid_fetch,
    "formation delete stubb --fetch",
    default_deployed_formation()
);

// Same dance as above, but by OID instead of name
test_fn_delete!(
    oid_has_local_oid,
    "formation delete frm-bcbdixdcojdu3o67lbh2gflaxe",
    default_deployed_formation()
);
test_fn_delete!(@no_name
    oid_no_local_oid,
    "formation delete frm-bcbdixdcojdu3o67lbh2gflaxe",
    default_local_formation()
);
// using --fetch and already knowing the OID should be no problem
test_fn_delete!(
    oid_no_local_oid_fetch,
    "formation delete frm-bcbdixdcojdu3o67lbh2gflaxe --fetch",
    default_deployed_formation()
);

// make sure we don't touch the API when --no-remote is set
test_fn_delete!(
    name_has_local_oid_no_remote,
    "formation delete stubb --no-remote",
    default_deployed_formation()
);
