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

// This could be a fn and not a macro...but as a macro we don't have to worry about return values
// and other things. And since this is just a test the macro is easier.
#[macro_export]
macro_rules! mock_fetch {
    (@impl $path:expr, $ret:expr) => {{
        MOCK_SERVER.mock(|w, then| {
            when(w, GET, $path);
            then.status(200)
                .json_body_obj(&serde_json::to_value($ret).unwrap());
        })
    }};
    ($oid:expr) => {{
        mock_fetch!(@impl format!("/v2beta/formations/{}", $oid), default_deployed_formation())
    }};
    () => {{
        let pr = PagedResponse {
            objects: multi_deployed_formations(),
            meta: PageMetadata {
                total: 1,
                next: None,
                prev: None,
            }
        };

        mock_fetch!(@impl "/v2beta/formations", pr)
    }};
}

macro_rules! test_fn_fetch {
    (@impl $test_fn:ident, $argv:expr, $frm:expr, $err:expr, $correct_out:expr, $MOCK:expr) => {
        #[test]
        fn $test_fn() {
            let mut mock = $MOCK;
            let res = test_main_exec_with_ctx(&argv!($argv), ctx_with_formations(vec![$frm]));
            if $err {
                assert!(res.is_err(), "{res:?}");
                mock.assert_hits(0);
            } else {
                assert!(res.is_ok(), "{res:?}");
                mock.assert();
                let actual_out: String = printer().as_string().trim().to_string();
                assert_eq!(
                    $correct_out,
                    &actual_out
                );
            }

            printer().clear();
            mock.delete();
        }
    };
    (@is_err $test_fn:ident, $argv:expr, $oid:expr, $local_db:expr) => {
        test_fn_fetch!(@impl
                       $test_fn,
                       $argv,
                       $local_db,
                       true,
                       "Successfully fetched Formation Instance stubb \
                        (frm-bcbdixdcojdu3o67lbh2gflaxe)",
                       mock_fetch!($oid));
    };
    ($test_fn:ident, $argv:expr, $oid:expr, $local_db:expr) => {
        test_fn_fetch!(@impl
                       $test_fn,
                       $argv,
                       $local_db,
                       false,
                       "Successfully fetched Formation Instance stubb \
                        (frm-bcbdixdcojdu3o67lbh2gflaxe)",
                       mock_fetch!($oid));
    };
    (@all $test_fn:ident, $argv:expr, $local_db:expr) => {
        test_fn_fetch!(@impl
                       $test_fn,
                       $argv,
                       $local_db,
                       false,
                       "Successfully fetched Formation Instance stubb \
                        (frm-bcbdixdcojdu3o67lbh2gflaxe)\n\
                        Successfully fetched Formation Instance stubb2 \
                        (frm-yenvkuety5fonocolcebsac6cy)",
                       mock_fetch!());
    };
}

// To be successful asking for a formation by name requires we already know it's OID which is why
// we use `default_deployed_formation` which has OIDs already populated
test_fn_fetch!(
    one_name_has_oid,
    "formation fetch-remote stubb",
    "frm-bcbdixdcojdu3o67lbh2gflaxe",
    default_deployed_formation()
);
// If we don't already have OIDs populated we should get a CLI error saying we dont know the OID of
// name "stubb"
test_fn_fetch!(
    @is_err
    one_name_no_oid,
    "formation fetch-remote stubb",
    "frm-bcbdixdcojdu3o67lbh2gflaxe",
    default_local_formation()
);

test_fn_fetch!(
    one_oid,
    "formation fetch-remote frm-bcbdixdcojdu3o67lbh2gflaxe",
    "frm-bcbdixdcojdu3o67lbh2gflaxe",
    default_deployed_formation()
);
// since we're asking for a specific OID it shouldn't matter if we don't know any local OIDs
test_fn_fetch!(
    one_oid_no_oids,
    "formation fetch-remote frm-bcbdixdcojdu3o67lbh2gflaxe",
    "frm-bcbdixdcojdu3o67lbh2gflaxe",
    default_local_formation()
);

// Likewise, asking for everything shouldn't depend on if we locally know what the OIDs are or not
test_fn_fetch!(@all all, "formation fetch-remote", default_deployed_formation());
test_fn_fetch!(@all
    all_oid_no_oids,
    "formation fetch-remote",
    default_local_formation()
);
