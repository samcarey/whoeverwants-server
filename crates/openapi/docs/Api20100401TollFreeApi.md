# \Api20100401TollFreeApi

All URIs are relative to *https://api.twilio.com*

Method | HTTP request | Description
------------- | ------------- | -------------
[**create_incoming_phone_number_toll_free**](Api20100401TollFreeApi.md#create_incoming_phone_number_toll_free) | **POST** /2010-04-01/Accounts/{AccountSid}/IncomingPhoneNumbers/TollFree.json | 
[**list_available_phone_number_toll_free**](Api20100401TollFreeApi.md#list_available_phone_number_toll_free) | **GET** /2010-04-01/Accounts/{AccountSid}/AvailablePhoneNumbers/{CountryCode}/TollFree.json | 
[**list_incoming_phone_number_toll_free**](Api20100401TollFreeApi.md#list_incoming_phone_number_toll_free) | **GET** /2010-04-01/Accounts/{AccountSid}/IncomingPhoneNumbers/TollFree.json | 



## create_incoming_phone_number_toll_free

> crate::models::ApiPeriodV2010PeriodAccountPeriodIncomingPhoneNumberPeriodIncomingPhoneNumberTollFree create_incoming_phone_number_toll_free(account_sid, phone_number, api_version, friendly_name, sms_application_sid, sms_fallback_method, sms_fallback_url, sms_method, sms_url, status_callback, status_callback_method, voice_application_sid, voice_caller_id_lookup, voice_fallback_method, voice_fallback_url, voice_method, voice_url, identity_sid, address_sid, emergency_status, emergency_address_sid, trunk_sid, voice_receive_mode, bundle_sid)




### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_sid** | **String** | The SID of the [Account](https://www.twilio.com/docs/iam/api/account) that will create the resource. | [required] |
**phone_number** | **String** | The phone number to purchase specified in [E.164](https://www.twilio.com/docs/glossary/what-e164) format.  E.164 phone numbers consist of a + followed by the country code and subscriber number without punctuation characters. For example, +14155551234. | [required] |
**api_version** | Option<**String**> | The API version to use for incoming calls made to the new phone number. The default is `2010-04-01`. |  |
**friendly_name** | Option<**String**> | A descriptive string that you created to describe the new phone number. It can be up to 64 characters long. By default, this is a formatted version of the phone number. |  |
**sms_application_sid** | Option<**String**> | The SID of the application that should handle SMS messages sent to the new phone number. If an `sms_application_sid` is present, we ignore all `sms_*_url` values and use those of the application. |  |
**sms_fallback_method** | Option<**String**> | The HTTP method that we should use to call `sms_fallback_url`. Can be: `GET` or `POST` and defaults to `POST`. |  |
**sms_fallback_url** | Option<**String**> | The URL that we should call when an error occurs while requesting or executing the TwiML defined by `sms_url`. |  |
**sms_method** | Option<**String**> | The HTTP method that we should use to call `sms_url`. Can be: `GET` or `POST` and defaults to `POST`. |  |
**sms_url** | Option<**String**> | The URL we should call when the new phone number receives an incoming SMS message. |  |
**status_callback** | Option<**String**> | The URL we should call using the `status_callback_method` to send status information to your application. |  |
**status_callback_method** | Option<**String**> | The HTTP method we should use to call `status_callback`. Can be: `GET` or `POST` and defaults to `POST`. |  |
**voice_application_sid** | Option<**String**> | The SID of the application we should use to handle calls to the new phone number. If a `voice_application_sid` is present, we ignore all of the voice urls and use those set on the application. Setting a `voice_application_sid` will automatically delete your `trunk_sid` and vice versa. |  |
**voice_caller_id_lookup** | Option<**bool**> | Whether to lookup the caller's name from the CNAM database and post it to your app. Can be: `true` or `false` and defaults to `false`. |  |
**voice_fallback_method** | Option<**String**> | The HTTP method that we should use to call `voice_fallback_url`. Can be: `GET` or `POST` and defaults to `POST`. |  |
**voice_fallback_url** | Option<**String**> | The URL that we should call when an error occurs retrieving or executing the TwiML requested by `url`. |  |
**voice_method** | Option<**String**> | The HTTP method that we should use to call `voice_url`. Can be: `GET` or `POST` and defaults to `POST`. |  |
**voice_url** | Option<**String**> | The URL that we should call to answer a call to the new phone number. The `voice_url` will not be called if a `voice_application_sid` or a `trunk_sid` is set. |  |
**identity_sid** | Option<**String**> | The SID of the Identity resource that we should associate with the new phone number. Some regions require an Identity to meet local regulations. |  |
**address_sid** | Option<**String**> | The SID of the Address resource we should associate with the new phone number. Some regions require addresses to meet local regulations. |  |
**emergency_status** | Option<**crate::models::IncomingPhoneNumberTollFreeEnumEmergencyStatus**> |  |  |
**emergency_address_sid** | Option<**String**> | The SID of the emergency address configuration to use for emergency calling from the new phone number. |  |
**trunk_sid** | Option<**String**> | The SID of the Trunk we should use to handle calls to the new phone number. If a `trunk_sid` is present, we ignore all of the voice urls and voice applications and use only those set on the Trunk. Setting a `trunk_sid` will automatically delete your `voice_application_sid` and vice versa. |  |
**voice_receive_mode** | Option<**crate::models::IncomingPhoneNumberTollFreeEnumVoiceReceiveMode**> |  |  |
**bundle_sid** | Option<**String**> | The SID of the Bundle resource that you associate with the phone number. Some regions require a Bundle to meet local Regulations. |  |

### Return type

[**crate::models::ApiPeriodV2010PeriodAccountPeriodIncomingPhoneNumberPeriodIncomingPhoneNumberTollFree**](api.v2010.account.incoming_phone_number.incoming_phone_number_toll_free.md)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: application/x-www-form-urlencoded
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## list_available_phone_number_toll_free

> crate::models::ListAvailablePhoneNumberTollFreeResponse list_available_phone_number_toll_free(account_sid, country_code, area_code, contains, sms_enabled, mms_enabled, voice_enabled, exclude_all_address_required, exclude_local_address_required, exclude_foreign_address_required, beta, near_number, near_lat_long, distance, in_postal_code, in_region, in_rate_center, in_lata, in_locality, fax_enabled, page_size, page, page_token)




### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_sid** | **String** | The SID of the [Account](https://www.twilio.com/docs/iam/api/account) requesting the AvailablePhoneNumber resources. | [required] |
**country_code** | **String** | The [ISO-3166-1](https://en.wikipedia.org/wiki/ISO_3166-1_alpha-2) country code of the country from which to read phone numbers. | [required] |
**area_code** | Option<**i32**> | The area code of the phone numbers to read. Applies to only phone numbers in the US and Canada. |  |
**contains** | Option<**String**> | The pattern on which to match phone numbers. Valid characters are `*`, `0-9`, `a-z`, and `A-Z`. The `*` character matches any single digit. For examples, see [Example 2](https://www.twilio.com/docs/phone-numbers/api/availablephonenumber-resource#local-get-basic-example-2) and [Example 3](https://www.twilio.com/docs/phone-numbers/api/availablephonenumber-resource#local-get-basic-example-3). If specified, this value must have at least two characters. |  |
**sms_enabled** | Option<**bool**> | Whether the phone numbers can receive text messages. Can be: `true` or `false`. |  |
**mms_enabled** | Option<**bool**> | Whether the phone numbers can receive MMS messages. Can be: `true` or `false`. |  |
**voice_enabled** | Option<**bool**> | Whether the phone numbers can receive calls. Can be: `true` or `false`. |  |
**exclude_all_address_required** | Option<**bool**> | Whether to exclude phone numbers that require an [Address](https://www.twilio.com/docs/usage/api/address). Can be: `true` or `false` and the default is `false`. |  |
**exclude_local_address_required** | Option<**bool**> | Whether to exclude phone numbers that require a local [Address](https://www.twilio.com/docs/usage/api/address). Can be: `true` or `false` and the default is `false`. |  |
**exclude_foreign_address_required** | Option<**bool**> | Whether to exclude phone numbers that require a foreign [Address](https://www.twilio.com/docs/usage/api/address). Can be: `true` or `false` and the default is `false`. |  |
**beta** | Option<**bool**> | Whether to read phone numbers that are new to the Twilio platform. Can be: `true` or `false` and the default is `true`. |  |
**near_number** | Option<**String**> | Given a phone number, find a geographically close number within `distance` miles. Distance defaults to 25 miles. Applies to only phone numbers in the US and Canada. |  |
**near_lat_long** | Option<**String**> | Given a latitude/longitude pair `lat,long` find geographically close numbers within `distance` miles. Applies to only phone numbers in the US and Canada. |  |
**distance** | Option<**i32**> | The search radius, in miles, for a `near_` query.  Can be up to `500` and the default is `25`. Applies to only phone numbers in the US and Canada. |  |
**in_postal_code** | Option<**String**> | Limit results to a particular postal code. Given a phone number, search within the same postal code as that number. Applies to only phone numbers in the US and Canada. |  |
**in_region** | Option<**String**> | Limit results to a particular region, state, or province. Given a phone number, search within the same region as that number. Applies to only phone numbers in the US and Canada. |  |
**in_rate_center** | Option<**String**> | Limit results to a specific rate center, or given a phone number search within the same rate center as that number. Requires `in_lata` to be set as well. Applies to only phone numbers in the US and Canada. |  |
**in_lata** | Option<**String**> | Limit results to a specific local access and transport area ([LATA](https://en.wikipedia.org/wiki/Local_access_and_transport_area)). Given a phone number, search within the same [LATA](https://en.wikipedia.org/wiki/Local_access_and_transport_area) as that number. Applies to only phone numbers in the US and Canada. |  |
**in_locality** | Option<**String**> | Limit results to a particular locality or city. Given a phone number, search within the same Locality as that number. |  |
**fax_enabled** | Option<**bool**> | Whether the phone numbers can receive faxes. Can be: `true` or `false`. |  |
**page_size** | Option<**i32**> | How many resources to return in each list page. The default is 50, and the maximum is 1000. |  |
**page** | Option<**i32**> | The page index. This value is simply for client state. |  |
**page_token** | Option<**String**> | The page token. This is provided by the API. |  |

### Return type

[**crate::models::ListAvailablePhoneNumberTollFreeResponse**](ListAvailablePhoneNumberTollFreeResponse.md)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## list_incoming_phone_number_toll_free

> crate::models::ListIncomingPhoneNumberTollFreeResponse list_incoming_phone_number_toll_free(account_sid, beta, friendly_name, phone_number, origin, page_size, page, page_token)




### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_sid** | **String** | The SID of the [Account](https://www.twilio.com/docs/iam/api/account) that created the resources to read. | [required] |
**beta** | Option<**bool**> | Whether to include phone numbers new to the Twilio platform. Can be: `true` or `false` and the default is `true`. |  |
**friendly_name** | Option<**String**> | A string that identifies the resources to read. |  |
**phone_number** | Option<**String**> | The phone numbers of the IncomingPhoneNumber resources to read. You can specify partial numbers and use '*' as a wildcard for any digit. |  |
**origin** | Option<**String**> | Whether to include phone numbers based on their origin. Can be: `twilio` or `hosted`. By default, phone numbers of all origin are included. |  |
**page_size** | Option<**i32**> | How many resources to return in each list page. The default is 50, and the maximum is 1000. |  |
**page** | Option<**i32**> | The page index. This value is simply for client state. |  |
**page_token** | Option<**String**> | The page token. This is provided by the API. |  |

### Return type

[**crate::models::ListIncomingPhoneNumberTollFreeResponse**](ListIncomingPhoneNumberTollFreeResponse.md)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

