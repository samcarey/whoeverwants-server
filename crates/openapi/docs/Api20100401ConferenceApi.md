# \Api20100401ConferenceApi

All URIs are relative to *https://api.twilio.com*

Method | HTTP request | Description
------------- | ------------- | -------------
[**fetch_conference**](Api20100401ConferenceApi.md#fetch_conference) | **GET** /2010-04-01/Accounts/{AccountSid}/Conferences/{Sid}.json | 
[**list_conference**](Api20100401ConferenceApi.md#list_conference) | **GET** /2010-04-01/Accounts/{AccountSid}/Conferences.json | 
[**update_conference**](Api20100401ConferenceApi.md#update_conference) | **POST** /2010-04-01/Accounts/{AccountSid}/Conferences/{Sid}.json | 



## fetch_conference

> crate::models::ApiPeriodV2010PeriodAccountPeriodConference fetch_conference(account_sid, sid)


Fetch an instance of a conference

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_sid** | **String** | The SID of the [Account](https://www.twilio.com/docs/iam/api/account) that created the Conference resource(s) to fetch. | [required] |
**sid** | **String** | The Twilio-provided string that uniquely identifies the Conference resource to fetch | [required] |

### Return type

[**crate::models::ApiPeriodV2010PeriodAccountPeriodConference**](api.v2010.account.conference.md)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## list_conference

> crate::models::ListConferenceResponse list_conference(account_sid, date_created, date_created_less_than, date_created_greater_than, date_updated, date_updated_less_than, date_updated_greater_than, friendly_name, status, page_size, page, page_token)


Retrieve a list of conferences belonging to the account used to make the request

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_sid** | **String** | The SID of the [Account](https://www.twilio.com/docs/iam/api/account) that created the Conference resource(s) to read. | [required] |
**date_created** | Option<**String**> | The `date_created` value, specified as `YYYY-MM-DD`, of the resources to read. To read conferences that started on or before midnight on a date, use `<=YYYY-MM-DD`, and to specify  conferences that started on or after midnight on a date, use `>=YYYY-MM-DD`. |  |
**date_created_less_than** | Option<**String**> | The `date_created` value, specified as `YYYY-MM-DD`, of the resources to read. To read conferences that started on or before midnight on a date, use `<=YYYY-MM-DD`, and to specify  conferences that started on or after midnight on a date, use `>=YYYY-MM-DD`. |  |
**date_created_greater_than** | Option<**String**> | The `date_created` value, specified as `YYYY-MM-DD`, of the resources to read. To read conferences that started on or before midnight on a date, use `<=YYYY-MM-DD`, and to specify  conferences that started on or after midnight on a date, use `>=YYYY-MM-DD`. |  |
**date_updated** | Option<**String**> | The `date_updated` value, specified as `YYYY-MM-DD`, of the resources to read. To read conferences that were last updated on or before midnight on a date, use `<=YYYY-MM-DD`, and to specify conferences that were last updated on or after midnight on a given date, use  `>=YYYY-MM-DD`. |  |
**date_updated_less_than** | Option<**String**> | The `date_updated` value, specified as `YYYY-MM-DD`, of the resources to read. To read conferences that were last updated on or before midnight on a date, use `<=YYYY-MM-DD`, and to specify conferences that were last updated on or after midnight on a given date, use  `>=YYYY-MM-DD`. |  |
**date_updated_greater_than** | Option<**String**> | The `date_updated` value, specified as `YYYY-MM-DD`, of the resources to read. To read conferences that were last updated on or before midnight on a date, use `<=YYYY-MM-DD`, and to specify conferences that were last updated on or after midnight on a given date, use  `>=YYYY-MM-DD`. |  |
**friendly_name** | Option<**String**> | The string that identifies the Conference resources to read. |  |
**status** | Option<**ConferenceEnumStatus**> | The status of the resources to read. Can be: `init`, `in-progress`, or `completed`. |  |
**page_size** | Option<**i32**> | How many resources to return in each list page. The default is 50, and the maximum is 1000. |  |
**page** | Option<**i32**> | The page index. This value is simply for client state. |  |
**page_token** | Option<**String**> | The page token. This is provided by the API. |  |

### Return type

[**crate::models::ListConferenceResponse**](ListConferenceResponse.md)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## update_conference

> crate::models::ApiPeriodV2010PeriodAccountPeriodConference update_conference(account_sid, sid, status, announce_url, announce_method)




### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_sid** | **String** | The SID of the [Account](https://www.twilio.com/docs/iam/api/account) that created the Conference resource(s) to update. | [required] |
**sid** | **String** | The Twilio-provided string that uniquely identifies the Conference resource to update | [required] |
**status** | Option<**crate::models::ConferenceEnumUpdateStatus**> |  |  |
**announce_url** | Option<**String**> | The URL we should call to announce something into the conference. The URL may return an MP3 file, a WAV file, or a TwiML document that contains `<Play>`, `<Say>`, `<Pause>`, or `<Redirect>` verbs. |  |
**announce_method** | Option<**String**> | The HTTP method used to call `announce_url`. Can be: `GET` or `POST` and the default is `POST` |  |

### Return type

[**crate::models::ApiPeriodV2010PeriodAccountPeriodConference**](api.v2010.account.conference.md)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: application/x-www-form-urlencoded
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

