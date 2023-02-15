use httpmock::prelude::*;
use seaplane_cli::{printer::printer, test_main_exec_with_ctx};

use crate::api::{
    formation::{ctx_with_formations, default_deployed_formation, default_local_formation},
    when, MOCK_SERVER,
};

macro_rules! mock_launch {
    ($argv:expr, $ctx:expr, $correct_out:expr, $hit:expr) => {{
        let mut mock_launch = MOCK_SERVER.mock(|w, then| {
            when(w, POST, "/v2beta/formations");
            then.status(201)
                .json_body(serde_json::to_value(default_deployed_formation()).unwrap());
        });
        let res = test_main_exec_with_ctx(&argv!($argv), $ctx);
        assert!(res.is_ok(), "{res:?}");

        if $hit {
            mock_launch.assert();
            let actual_out: String = printer().as_string().trim().to_string();
            assert_eq!($correct_out, actual_out);
        } else {
            mock_launch.assert_hits(0);
        }
        mock_launch.delete();
    }};
}

#[test]
fn name() {
    mock_launch!(
        "formation launch stubb",
        ctx_with_formations(vec![default_local_formation()]),
        "Successfully Launched remote Formation Instance stubb (frm-bcbdixdcojdu3o67lbh2gflaxe)\n\
        Formation Instance URL is: https://stubb.tenant.on.cplane.cloud/\n\
        (hint: it may take up to a minute for the Formation to become fully online)\n\
        (hint: check the status of this Formation Instance with 'seaplane formation status stubb')",
        true
    );
    printer().clear();
}
