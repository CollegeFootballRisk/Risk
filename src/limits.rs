use rocket_governor::{Method, Quota, RocketGovernable, RocketGovernor};
pub struct RateLimitGuard;
impl<'r> RocketGovernable<'r> for RateLimitGuard {
    fn quota(_method: Method, _route_name: &str) -> Quota {
        // Set quota to 10 requests per second
        Quota::per_second(Self::nonzero(10u32))
    }
}
