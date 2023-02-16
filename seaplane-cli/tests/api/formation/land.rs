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

macro_rules! mock_land {
    ($argv:expr, $ctx:expr, $correct_out:expr, $err:expr, $hit:expr) => {{
        let mut mock_fetch = mock_fetch!();
        let mut mock_land = MOCK_SERVER.mock(|w, then| {
            when(w, DELETE, "/v2beta/formations/frm-euqecs8n6h5l552ps6skal12lc");
            then.status(200).body("success");
        });
        let res = test_main_exec_with_ctx(&argv!($argv), $ctx);
        if $err {
            assert!(res.is_err(), "{res:?}");
        } else {
            assert!(res.is_ok(), "{res:?}");
            if $argv.contains("--fetch") {
                mock_fetch.assert();
            } else {
                mock_fetch.assert_hits(0);
            }

            if $hit {
                mock_land.assert();
            } else {
                mock_land.assert_hits(0);
            }
            let actual_out: String = printer().as_string().trim().to_string();
            assert_eq!($correct_out, actual_out);
        }

        mock_fetch.delete();
        mock_land.delete();
    }};
}
macro_rules! test_fn_land {
    (@impl $test_fn:ident, $argv:expr, $ctx:expr, $correct_out:expr, $err:expr) => {
        #[test]
        fn $test_fn() {
            mock_land!($argv, $ctx, $correct_out, $err, true);
            printer().clear();
        }
    };
    (@is_err $test_fn:ident, $argv:expr, $local_db:expr) => {
        test_fn_land!(@impl
                        $test_fn,
                        $argv,
                        ctx_with_formations(vec![$local_db]),
                        "",
                        true);
    };
    ($test_fn:ident, $argv:expr, $local_db:expr) => {
        test_fn_land!(@impl
                        $test_fn,
                        $argv,
                        ctx_with_formations(vec![$local_db]),
                        "Successfully Landed remote Formation Instance \
                        frm-euqecs8n6h5l552ps6skal12lc (stubb)",
                        false);
    };
    ($test_fn:ident, $argv:expr, $local_db:expr, $correct_out:expr) => {
        test_fn_land!(@impl
                        $test_fn,
                        $argv,
                        ctx_with_formations(vec![$local_db]),
                        $correct_out,
                        false);
    };
}

// Land a single formation by name where we already know the OID
test_fn_land!(name_has_local_oid, "formation land stubb", default_deployed_formation());
// If we don't already have OIDs populated we should get a CLI error saying we dont know the OID of
// name "stubb"
test_fn_land!(
    @is_err
    name_no_local_oid,
    "formation land stubb",
    default_local_formation()
);
// --fetch fixes that
test_fn_land!(name_no_local_oid_fetch, "formation land stubb --fetch", default_local_formation());
// using --fetch and already knowing the OID should be no problem
test_fn_land!(
    name_has_local_oid_fetch,
    "formation land stubb --fetch",
    default_deployed_formation()
);

// Same dance as above, but by OID instead of name
test_fn_land!(
    oid_has_local_oid,
    "formation land frm-euqecs8n6h5l552ps6skal12lc",
    default_deployed_formation()
);
test_fn_land!(
    oid_no_local_oid,
    "formation land frm-euqecs8n6h5l552ps6skal12lc",
    default_local_formation(),
    "Successfully Landed remote Formation Instance frm-euqecs8n6h5l552ps6skal12lc"
);
// using --fetch and already knowing the OID should be no problem
test_fn_land!(
    oid_no_local_oid_fetch,
    "formation land frm-euqecs8n6h5l552ps6skal12lc --fetch",
    default_deployed_formation()
);
