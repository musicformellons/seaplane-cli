//! Note we only test `formation plan` combinations that touch the API
use httpmock::prelude::*;
use seaplane::api::compute::v2::{PageMetadata, PagedResponse};
use seaplane_cli::{printer::printer, test_main_exec_with_ctx};

use crate::api::{
    formation::{ctx_with_formations, default_deployed_formation, multi_deployed_formations},
    when, MOCK_SERVER,
};

macro_rules! test_fn_plan {
    (@impl $test_fn:ident, $argv:expr, $ctx:expr) => {
        #[test]
        fn $test_fn() {
            let mut mock_fetch = mock_fetch!();

            mock_launch!(
                $argv,
                $ctx,
        "Successfully created local Formation Plan stubb\n\
        Successfully Launched remote Formation Instance stubb (frm-euqecs8n6h5l552ps6skal12lc)\n\
        Formation Instance URL is: https://stubb.tenant.on.cplane.cloud/\n\
        (hint: it may take up to a minute for the Formation to become fully online)\n\
        (hint: check the status of this Formation Instance with 'seaplane formation status stubb')",
                $argv.contains("--launch"));

            if $argv.contains("--fetch") {
                mock_fetch.assert();
            } else {
                mock_fetch.assert_hits(0);
            }

            mock_fetch.delete();
            printer().clear();
        }
    };
    ($test_fn:ident, $argv:expr) => {
        test_fn_plan!(@impl
                        $test_fn,
                        $argv,
                        ctx_with_formations(Vec::new()));
    };
}

test_fn_plan!(
    one,
    "formation plan \
              --name stubb \
              --flight name=flask,image=foo.com/bar:latest \
              --flight name=pequod,image=foo.com/baz:latest \
              --gateway-flight flask \
              --launch"
);
test_fn_plan!(
    fetch_first,
    "formation plan \
              --name stubb \
              --flight name=flask,image=foo.com/bar:latest \
              --flight name=pequod,image=foo.com/baz:latest \
              --gateway-flight flask \
              --launch \
              --fetch"
);

// Test that no API is hit
test_fn_plan!(
    no_remote,
    "formation plan \
              --name stubb \
              --flight name=flask,image=foo.com/bar:latest \
              --flight name=pequod,image=foo.com/baz:latest \
              --gateway-flight flask"
);
