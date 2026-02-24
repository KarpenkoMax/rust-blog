use tonic::metadata::MetadataMap;
use tonic::{Code, Status};

use crate::infrastructure::jwt::JwtService;

#[derive(Debug, Clone)]
pub(crate) struct GrpcAuthContext {
    pub(crate) user_id: i64,
    pub(crate) username: String,
}

pub(crate) fn authenticate_request(
    jwt: &JwtService,
    metadata: &MetadataMap,
) -> Result<GrpcAuthContext, Status> {
    let token = parse_bearer_token(metadata)?;
    let claims = jwt
        .verify_token(token)
        .map_err(|_| Status::new(Code::Unauthenticated, "invalid token"))?;

    Ok(GrpcAuthContext {
        user_id: claims.user_id,
        username: claims.username,
    })
}

fn parse_bearer_token(metadata: &MetadataMap) -> Result<&str, Status> {
    let raw = metadata
        .get("authorization")
        .ok_or_else(|| Status::new(Code::Unauthenticated, "missing authorization metadata"))?;

    let raw = raw
        .to_str()
        .map_err(|_| Status::new(Code::Unauthenticated, "invalid authorization metadata"))?;

    let mut parts = raw.split_whitespace();
    let scheme = parts
        .next()
        .ok_or_else(|| Status::new(Code::Unauthenticated, "invalid authorization metadata"))?;
    let token = parts
        .next()
        .ok_or_else(|| Status::new(Code::Unauthenticated, "invalid authorization metadata"))?;

    if parts.next().is_some() {
        return Err(Status::new(
            Code::Unauthenticated,
            "invalid authorization metadata",
        ));
    }
    if !scheme.eq_ignore_ascii_case("bearer") || token.trim().is_empty() {
        return Err(Status::new(
            Code::Unauthenticated,
            "invalid authorization metadata",
        ));
    }

    Ok(token.trim())
}
