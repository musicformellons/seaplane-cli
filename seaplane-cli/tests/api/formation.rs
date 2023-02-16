#[macro_use]
mod fetch;
#[macro_use]
mod land; // uses fetch
mod delete; // uses land, fetch
#[macro_use]
mod launch;
mod plan; // uses launch, fetch
mod status; // uses fetch

use seaplane::api::compute::v2::{
    Flight as FlightModel, FlightStatus, Formation as FormationModel,
};
use seaplane_cli::{context::Ctx, ops::formation::Formation};

use crate::api::{ctx_from_url, MOCK_SERVER};

fn ctx_with_formations(models: Vec<FormationModel>) -> Ctx {
    let mut ctx = ctx_from_url(MOCK_SERVER.base_url());
    for model in models.into_iter() {
        let deployed = model.oid.is_some();
        let f = Formation { model, local: true, deployed };
        ctx.db.formations.inner.push(f);
    }
    ctx
}

fn local_formation(name: &str, f1: &str, f2: &str) -> FormationModel {
    FormationModel {
        name: name.into(),
        oid: None,
        url: None,
        flights: vec![
            FlightModel::builder()
                .name(f1)
                .image("foo.com/bar:latest")
                .build()
                .unwrap(),
            FlightModel::builder()
                .name(f2)
                .image("foo.com/baz:latest")
                .build()
                .unwrap(),
        ],
        gateway_flight: Some(f1.into()),
    }
}
fn default_local_formation() -> FormationModel { local_formation("stubb", "flask", "pequod") }

fn deployed_formation(
    formation: &mut FormationModel,
    oid: &str,
    url: &str,
    (f1_name, f1_oid): (&str, &str),
    (f2_name, f2_oid): (&str, &str),
) {
    formation.oid = Some(oid.parse().unwrap());
    formation.url = Some(url.parse().unwrap());
    if let Some(f1) = formation.flights.iter_mut().find(|f| f.name == f1_name) {
        f1.oid = Some(f1_oid.parse().unwrap());
        f1.status = FlightStatus::Healthy;
    }
    if let Some(f2) = formation.flights.iter_mut().find(|f| f.name == f2_name) {
        f2.oid = Some(f2_oid.parse().unwrap());
        f2.status = FlightStatus::Healthy;
    }
}

fn default_deployed_formation() -> FormationModel {
    let mut formation = default_local_formation();
    deployed_formation(
        &mut formation,
        "frm-euqecs8n6h5l552ps6skal12lc",
        "https://stubb.tenant.on.cplane.cloud",
        ("flask", "flt-dkk055g7qp5cdfr1ui9rc363dc"),
        ("pequod", "flt-va5g8orh1503ja6r9mqf1gvvj8"),
    );
    formation
}
fn multi_deployed_formations() -> Vec<FormationModel> {
    let mut f2 = local_formation("stubb2", "flask2", "pequod2");
    deployed_formation(
        &mut f2,
        "frm-45rt0pqpo93q78t6au6dd8kn04",
        "https://stubb2.tenant.on.cplane.cloud",
        ("flask2", "flt-r5mrglbl0d4qjeopdh8vli89oo"),
        ("pequod2", "flt-90mifnt75d4i3a6eevt4po2f7k"),
    );
    vec![default_deployed_formation(), f2]
}
