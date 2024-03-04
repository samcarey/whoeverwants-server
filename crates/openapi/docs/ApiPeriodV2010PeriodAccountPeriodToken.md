# ApiPeriodV2010PeriodAccountPeriodToken

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**account_sid** | Option<**String**> | The SID of the [Account](https://www.twilio.com/docs/iam/api/account) that created the Token resource. | [optional]
**date_created** | Option<**String**> | The date and time in GMT that the resource was created specified in [RFC 2822](https://www.ietf.org/rfc/rfc2822.txt) format. | [optional]
**date_updated** | Option<**String**> | The date and time in GMT that the resource was last updated specified in [RFC 2822](https://www.ietf.org/rfc/rfc2822.txt) format. | [optional]
**ice_servers** | Option<[**Vec<crate::models::ApiV2010AccountTokenIceServersInner>**](api_v2010_account_token_ice_servers_inner.md)> | An array representing the ephemeral credentials and the STUN and TURN server URIs. | [optional]
**password** | Option<**String**> | The temporary password that the username will use when authenticating with Twilio. | [optional]
**ttl** | Option<**String**> | The duration in seconds for which the username and password are valid. | [optional]
**username** | Option<**String**> | The temporary username that uniquely identifies a Token. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


