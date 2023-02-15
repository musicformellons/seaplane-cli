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

const STARTING: &str = r#"◉ Formation stubb: Starting
└─┐
  │   FLIGHT    STATUS      OID
  ├─◉ flask     Starting
  └─◉ pequod    Starting"#;

const ONE: &str = r#"◉ Formation stubb (frm-bcbdixdcojdu3o67lbh2gflaxe): Healthy
└─┐
  │   FLIGHT    STATUS     OID
  ├─◉ flask     Healthy    flt-kr7dkiqwbrf35frwkm7vxsghci
  └─◉ pequod    Healthy    flt-h7qvwdgh3fhwrm3iinslthbf6u"#;

const MULTI: &str = r#"◉ Formation stubb (frm-bcbdixdcojdu3o67lbh2gflaxe): Healthy
└─┐
  │   FLIGHT    STATUS     OID
  ├─◉ flask     Healthy    flt-kr7dkiqwbrf35frwkm7vxsghci
  └─◉ pequod    Healthy    flt-h7qvwdgh3fhwrm3iinslthbf6u
◉ Formation stubb2 (frm-yenvkuety5fonocolcebsac6cy): Healthy
└─┐
  │   FLIGHT     STATUS     OID
  ├─◉ flask2     Healthy    flt-hpzxknhkzfczxnrkzsd54cohxq
  └─◉ pequod2    Healthy    flt-i3hg6c3xfbdaxbpnotcqesulfe"#;

macro_rules! test_fn_status {
    (@impl $test_fn:ident, $argv:expr, $ctx:expr, $correct_out:expr, $MOCK:expr) => {
        #[test]
        fn $test_fn() {
            let mut mock = $MOCK;
            let res = test_main_exec_with_ctx(&argv!($argv), $ctx);
            assert!(res.is_ok(), "{res:?}");
            if $argv.contains("--no-fetch") {
                mock.assert_hits(0);
            } else {
                mock.assert();
            }
            let actual_out: String = printer().as_string().trim().to_string();
            assert_eq!($correct_out, actual_out);

            printer().clear();
            mock.delete();
        }
    };
    ($test_fn:ident, $argv:expr, $local_db:expr, $correct_out:expr) => {
        test_fn_status!(@impl
                        $test_fn,
                        $argv,
                        ctx_with_formations(vec![$local_db]),
                        $correct_out,
                        mock_fetch!());
    };
    (@all $test_fn:ident, $argv:expr, $local_db:expr, $correct_out:expr) => {
        test_fn_status!(@impl
                        $test_fn,
                        $argv,
                        ctx_with_formations($local_db),
                        $correct_out,
                        mock_fetch!());
    };
}

// Ask for single formation by name
test_fn_status!(one_name_has_oid, "formation status stubb", default_deployed_formation(), ONE);
test_fn_status!(
    one_name_has_oid_no_fetch,
    "formation status stubb --no-fetch",
    default_deployed_formation(),
    ONE
);
test_fn_status!(one_name_no_local_oid, "formation status stubb", default_local_formation(), ONE);
test_fn_status!(
    one_name_no_local_oid_no_fetch,
    "formation status stubb --no-fetch",
    default_local_formation(),
    STARTING
);

// Ask for single formation by OID
test_fn_status!(
    one_oid_has_oid,
    "formation status frm-bcbdixdcojdu3o67lbh2gflaxe",
    default_deployed_formation(),
    ONE
);
test_fn_status!(
    one_oid_has_oid_no_fetch,
    "formation status frm-bcbdixdcojdu3o67lbh2gflaxe --no-fetch",
    default_deployed_formation(),
    ONE
);
test_fn_status!(
    one_oid_no_local_oid,
    "formation status frm-bcbdixdcojdu3o67lbh2gflaxe",
    default_local_formation(),
    ONE
);
test_fn_status!(
    one_oid_no_local_oid_no_fetch,
    "formation status frm-bcbdixdcojdu3o67lbh2gflaxe --no-fetch",
    default_local_formation(),
    STARTING
);

// Ask for multiple formations by name
test_fn_status!(
    @all
    all,
    "formation status",
    multi_deployed_formations(),
    MULTI
);
test_fn_status!(
    @all
    all_no_fetch,
    "formation status --no-fetch",
    multi_deployed_formations(),
    MULTI
);
