//! `NetworkComponentNegotiator.negotiate`: which channels both sides use, or why they can't talk.

use crate::wire::Component;

/// The channels both sides agreed on, as (id, version), or the reasons per channel id.
pub type Negotiation = Result<Vec<(String, String)>, Vec<(String, String)>>;

/// The channels both sides use, or why they can't talk.
///
/// Optional channels the other side doesn't have are dropped first; every remaining channel
/// must then exist on both sides with the same flow (when given) and version. The server's
/// version is the chosen one.
pub fn negotiate(server: &[Component], client: &[Component]) -> Negotiation {
    let has = |side: &[&Component], id: &str| side.iter().any(|c| c.id == id);
    let server_all: Vec<&Component> = server.iter().collect();
    let client: Vec<&Component> = client
        .iter()
        .filter(|c| !c.optional || has(&server_all, &c.id))
        .collect();
    let server: Vec<&Component> = server
        .iter()
        .filter(|c| !c.optional || has(&client, &c.id))
        .collect();

    let missing_on_server: Vec<(String, String)> = client
        .iter()
        .filter(|c| !has(&server, &c.id))
        .map(|c| (c.id.clone(), "missing on the server".to_string()))
        .collect();
    if !missing_on_server.is_empty() {
        return Err(missing_on_server);
    }
    let missing_on_client: Vec<(String, String)> = server
        .iter()
        .filter(|c| !has(&client, &c.id))
        .map(|c| (c.id.clone(), "missing on the client".to_string()))
        .collect();
    if !missing_on_client.is_empty() {
        return Err(missing_on_client);
    }

    let mut agreed = Vec::new();
    let mut failures = Vec::new();
    for s in &server {
        let Some(c) = client.iter().find(|c| c.id == s.id) else {
            continue;
        };
        match validate(s, c).or_else(|| validate(c, s)) {
            Some(reason) => failures.push((s.id.clone(), reason)),
            None => agreed.push((s.id.clone(), s.version.clone())),
        }
    }
    if failures.is_empty() {
        Ok(agreed)
    } else {
        Err(failures)
    }
}

/// `validateComponent`: a failure reason, or none when `left` and `right` agree.
fn validate(left: &Component, right: &Component) -> Option<String> {
    if let Some(flow) = left.flow {
        match right.flow {
            None => return Some(format!("flow {flow:?} missing on the other side")),
            Some(other) if other != flow => {
                return Some(format!("flow {flow:?} against {other:?}"));
            }
            Some(_) => {}
        }
    }
    (left.version != right.version)
        .then(|| format!("version {} against {}", left.version, right.version))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wire::Flow;

    fn channel(id: &str, version: &str, optional: bool) -> Component {
        Component {
            id: id.into(),
            version: version.into(),
            flow: Some(Flow::Clientbound),
            optional,
        }
    }

    #[test]
    fn optional_channels_missing_on_one_side_are_dropped() {
        let server = [
            channel("a:shared", "1", true),
            channel("a:server_only", "1", true),
        ];
        let client = [
            channel("a:shared", "1", true),
            channel("a:client_only", "1", true),
        ];
        assert_eq!(
            negotiate(&server, &client),
            Ok(vec![("a:shared".into(), "1".into())])
        );
    }

    #[test]
    fn a_required_client_channel_missing_on_the_server_fails() {
        let client = [channel("mod:required", "1", false)];
        assert!(negotiate(&[], &client).is_err());
    }

    #[test]
    fn versions_must_match() {
        let server = [channel("a:b", "1", false)];
        let client = [channel("a:b", "2", false)];
        assert!(negotiate(&server, &client).is_err());
    }
}
