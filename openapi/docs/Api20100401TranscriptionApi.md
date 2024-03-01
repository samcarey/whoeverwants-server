# \Api20100401TranscriptionApi

All URIs are relative to *https://api.twilio.com*

Method | HTTP request | Description
------------- | ------------- | -------------
[**delete_recording_transcription**](Api20100401TranscriptionApi.md#delete_recording_transcription) | **DELETE** /2010-04-01/Accounts/{AccountSid}/Recordings/{RecordingSid}/Transcriptions/{Sid}.json | 
[**delete_transcription**](Api20100401TranscriptionApi.md#delete_transcription) | **DELETE** /2010-04-01/Accounts/{AccountSid}/Transcriptions/{Sid}.json | 
[**fetch_recording_transcription**](Api20100401TranscriptionApi.md#fetch_recording_transcription) | **GET** /2010-04-01/Accounts/{AccountSid}/Recordings/{RecordingSid}/Transcriptions/{Sid}.json | 
[**fetch_transcription**](Api20100401TranscriptionApi.md#fetch_transcription) | **GET** /2010-04-01/Accounts/{AccountSid}/Transcriptions/{Sid}.json | 
[**list_recording_transcription**](Api20100401TranscriptionApi.md#list_recording_transcription) | **GET** /2010-04-01/Accounts/{AccountSid}/Recordings/{RecordingSid}/Transcriptions.json | 
[**list_transcription**](Api20100401TranscriptionApi.md#list_transcription) | **GET** /2010-04-01/Accounts/{AccountSid}/Transcriptions.json | 



## delete_recording_transcription

> delete_recording_transcription(account_sid, recording_sid, sid)




### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_sid** | **String** | The SID of the [Account](https://www.twilio.com/docs/iam/api/account) that created the Transcription resources to delete. | [required] |
**recording_sid** | **String** | The SID of the [Recording](https://www.twilio.com/docs/voice/api/recording) that created the transcription to delete. | [required] |
**sid** | **String** | The Twilio-provided string that uniquely identifies the Transcription resource to delete. | [required] |

### Return type

 (empty response body)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## delete_transcription

> delete_transcription(account_sid, sid)


Delete a transcription from the account used to make the request

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_sid** | **String** | The SID of the [Account](https://www.twilio.com/docs/iam/api/account) that created the Transcription resources to delete. | [required] |
**sid** | **String** | The Twilio-provided string that uniquely identifies the Transcription resource to delete. | [required] |

### Return type

 (empty response body)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## fetch_recording_transcription

> crate::models::ApiPeriodV2010PeriodAccountPeriodRecordingPeriodRecordingTranscription fetch_recording_transcription(account_sid, recording_sid, sid)




### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_sid** | **String** | The SID of the [Account](https://www.twilio.com/docs/iam/api/account) that created the Transcription resource to fetch. | [required] |
**recording_sid** | **String** | The SID of the [Recording](https://www.twilio.com/docs/voice/api/recording) that created the transcription to fetch. | [required] |
**sid** | **String** | The Twilio-provided string that uniquely identifies the Transcription resource to fetch. | [required] |

### Return type

[**crate::models::ApiPeriodV2010PeriodAccountPeriodRecordingPeriodRecordingTranscription**](api.v2010.account.recording.recording_transcription.md)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## fetch_transcription

> crate::models::ApiPeriodV2010PeriodAccountPeriodTranscription fetch_transcription(account_sid, sid)


Fetch an instance of a Transcription

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_sid** | **String** | The SID of the [Account](https://www.twilio.com/docs/iam/api/account) that created the Transcription resource to fetch. | [required] |
**sid** | **String** | The Twilio-provided string that uniquely identifies the Transcription resource to fetch. | [required] |

### Return type

[**crate::models::ApiPeriodV2010PeriodAccountPeriodTranscription**](api.v2010.account.transcription.md)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## list_recording_transcription

> crate::models::ListRecordingTranscriptionResponse list_recording_transcription(account_sid, recording_sid, page_size, page, page_token)




### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_sid** | **String** | The SID of the [Account](https://www.twilio.com/docs/iam/api/account) that created the Transcription resources to read. | [required] |
**recording_sid** | **String** | The SID of the [Recording](https://www.twilio.com/docs/voice/api/recording) that created the transcriptions to read. | [required] |
**page_size** | Option<**i32**> | How many resources to return in each list page. The default is 50, and the maximum is 1000. |  |
**page** | Option<**i32**> | The page index. This value is simply for client state. |  |
**page_token** | Option<**String**> | The page token. This is provided by the API. |  |

### Return type

[**crate::models::ListRecordingTranscriptionResponse**](ListRecordingTranscriptionResponse.md)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## list_transcription

> crate::models::ListTranscriptionResponse list_transcription(account_sid, page_size, page, page_token)


Retrieve a list of transcriptions belonging to the account used to make the request

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_sid** | **String** | The SID of the [Account](https://www.twilio.com/docs/iam/api/account) that created the Transcription resources to read. | [required] |
**page_size** | Option<**i32**> | How many resources to return in each list page. The default is 50, and the maximum is 1000. |  |
**page** | Option<**i32**> | The page index. This value is simply for client state. |  |
**page_token** | Option<**String**> | The page token. This is provided by the API. |  |

### Return type

[**crate::models::ListTranscriptionResponse**](ListTranscriptionResponse.md)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

