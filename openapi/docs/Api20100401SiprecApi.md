# \Api20100401SiprecApi

All URIs are relative to *https://api.twilio.com*

Method | HTTP request | Description
------------- | ------------- | -------------
[**create_siprec**](Api20100401SiprecApi.md#create_siprec) | **POST** /2010-04-01/Accounts/{AccountSid}/Calls/{CallSid}/Siprec.json | 
[**update_siprec**](Api20100401SiprecApi.md#update_siprec) | **POST** /2010-04-01/Accounts/{AccountSid}/Calls/{CallSid}/Siprec/{Sid}.json | 



## create_siprec

> crate::models::ApiPeriodV2010PeriodAccountPeriodCallPeriodSiprec create_siprec(account_sid, call_sid, name, connector_name, track, status_callback, status_callback_method, parameter1_period_name, parameter1_period_value, parameter2_period_name, parameter2_period_value, parameter3_period_name, parameter3_period_value, parameter4_period_name, parameter4_period_value, parameter5_period_name, parameter5_period_value, parameter6_period_name, parameter6_period_value, parameter7_period_name, parameter7_period_value, parameter8_period_name, parameter8_period_value, parameter9_period_name, parameter9_period_value, parameter10_period_name, parameter10_period_value, parameter11_period_name, parameter11_period_value, parameter12_period_name, parameter12_period_value, parameter13_period_name, parameter13_period_value, parameter14_period_name, parameter14_period_value, parameter15_period_name, parameter15_period_value, parameter16_period_name, parameter16_period_value, parameter17_period_name, parameter17_period_value, parameter18_period_name, parameter18_period_value, parameter19_period_name, parameter19_period_value, parameter20_period_name, parameter20_period_value, parameter21_period_name, parameter21_period_value, parameter22_period_name, parameter22_period_value, parameter23_period_name, parameter23_period_value, parameter24_period_name, parameter24_period_value, parameter25_period_name, parameter25_period_value, parameter26_period_name, parameter26_period_value, parameter27_period_name, parameter27_period_value, parameter28_period_name, parameter28_period_value, parameter29_period_name, parameter29_period_value, parameter30_period_name, parameter30_period_value, parameter31_period_name, parameter31_period_value, parameter32_period_name, parameter32_period_value, parameter33_period_name, parameter33_period_value, parameter34_period_name, parameter34_period_value, parameter35_period_name, parameter35_period_value, parameter36_period_name, parameter36_period_value, parameter37_period_name, parameter37_period_value, parameter38_period_name, parameter38_period_value, parameter39_period_name, parameter39_period_value, parameter40_period_name, parameter40_period_value, parameter41_period_name, parameter41_period_value, parameter42_period_name, parameter42_period_value, parameter43_period_name, parameter43_period_value, parameter44_period_name, parameter44_period_value, parameter45_period_name, parameter45_period_value, parameter46_period_name, parameter46_period_value, parameter47_period_name, parameter47_period_value, parameter48_period_name, parameter48_period_value, parameter49_period_name, parameter49_period_value, parameter50_period_name, parameter50_period_value, parameter51_period_name, parameter51_period_value, parameter52_period_name, parameter52_period_value, parameter53_period_name, parameter53_period_value, parameter54_period_name, parameter54_period_value, parameter55_period_name, parameter55_period_value, parameter56_period_name, parameter56_period_value, parameter57_period_name, parameter57_period_value, parameter58_period_name, parameter58_period_value, parameter59_period_name, parameter59_period_value, parameter60_period_name, parameter60_period_value, parameter61_period_name, parameter61_period_value, parameter62_period_name, parameter62_period_value, parameter63_period_name, parameter63_period_value, parameter64_period_name, parameter64_period_value, parameter65_period_name, parameter65_period_value, parameter66_period_name, parameter66_period_value, parameter67_period_name, parameter67_period_value, parameter68_period_name, parameter68_period_value, parameter69_period_name, parameter69_period_value, parameter70_period_name, parameter70_period_value, parameter71_period_name, parameter71_period_value, parameter72_period_name, parameter72_period_value, parameter73_period_name, parameter73_period_value, parameter74_period_name, parameter74_period_value, parameter75_period_name, parameter75_period_value, parameter76_period_name, parameter76_period_value, parameter77_period_name, parameter77_period_value, parameter78_period_name, parameter78_period_value, parameter79_period_name, parameter79_period_value, parameter80_period_name, parameter80_period_value, parameter81_period_name, parameter81_period_value, parameter82_period_name, parameter82_period_value, parameter83_period_name, parameter83_period_value, parameter84_period_name, parameter84_period_value, parameter85_period_name, parameter85_period_value, parameter86_period_name, parameter86_period_value, parameter87_period_name, parameter87_period_value, parameter88_period_name, parameter88_period_value, parameter89_period_name, parameter89_period_value, parameter90_period_name, parameter90_period_value, parameter91_period_name, parameter91_period_value, parameter92_period_name, parameter92_period_value, parameter93_period_name, parameter93_period_value, parameter94_period_name, parameter94_period_value, parameter95_period_name, parameter95_period_value, parameter96_period_name, parameter96_period_value, parameter97_period_name, parameter97_period_value, parameter98_period_name, parameter98_period_value, parameter99_period_name, parameter99_period_value)


Create a Siprec

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_sid** | **String** | The SID of the [Account](https://www.twilio.com/docs/iam/api/account) that created this Siprec resource. | [required] |
**call_sid** | **String** | The SID of the [Call](https://www.twilio.com/docs/voice/api/call-resource) the Siprec resource is associated with. | [required] |
**name** | Option<**String**> | The user-specified name of this Siprec, if one was given when the Siprec was created. This may be used to stop the Siprec. |  |
**connector_name** | Option<**String**> | Unique name used when configuring the connector via Marketplace Add-on. |  |
**track** | Option<**crate::models::SiprecEnumTrack**> |  |  |
**status_callback** | Option<**String**> | Absolute URL of the status callback. |  |
**status_callback_method** | Option<**String**> | The http method for the status_callback (one of GET, POST). |  |
**parameter1_period_name** | Option<**String**> | Parameter name |  |
**parameter1_period_value** | Option<**String**> | Parameter value |  |
**parameter2_period_name** | Option<**String**> | Parameter name |  |
**parameter2_period_value** | Option<**String**> | Parameter value |  |
**parameter3_period_name** | Option<**String**> | Parameter name |  |
**parameter3_period_value** | Option<**String**> | Parameter value |  |
**parameter4_period_name** | Option<**String**> | Parameter name |  |
**parameter4_period_value** | Option<**String**> | Parameter value |  |
**parameter5_period_name** | Option<**String**> | Parameter name |  |
**parameter5_period_value** | Option<**String**> | Parameter value |  |
**parameter6_period_name** | Option<**String**> | Parameter name |  |
**parameter6_period_value** | Option<**String**> | Parameter value |  |
**parameter7_period_name** | Option<**String**> | Parameter name |  |
**parameter7_period_value** | Option<**String**> | Parameter value |  |
**parameter8_period_name** | Option<**String**> | Parameter name |  |
**parameter8_period_value** | Option<**String**> | Parameter value |  |
**parameter9_period_name** | Option<**String**> | Parameter name |  |
**parameter9_period_value** | Option<**String**> | Parameter value |  |
**parameter10_period_name** | Option<**String**> | Parameter name |  |
**parameter10_period_value** | Option<**String**> | Parameter value |  |
**parameter11_period_name** | Option<**String**> | Parameter name |  |
**parameter11_period_value** | Option<**String**> | Parameter value |  |
**parameter12_period_name** | Option<**String**> | Parameter name |  |
**parameter12_period_value** | Option<**String**> | Parameter value |  |
**parameter13_period_name** | Option<**String**> | Parameter name |  |
**parameter13_period_value** | Option<**String**> | Parameter value |  |
**parameter14_period_name** | Option<**String**> | Parameter name |  |
**parameter14_period_value** | Option<**String**> | Parameter value |  |
**parameter15_period_name** | Option<**String**> | Parameter name |  |
**parameter15_period_value** | Option<**String**> | Parameter value |  |
**parameter16_period_name** | Option<**String**> | Parameter name |  |
**parameter16_period_value** | Option<**String**> | Parameter value |  |
**parameter17_period_name** | Option<**String**> | Parameter name |  |
**parameter17_period_value** | Option<**String**> | Parameter value |  |
**parameter18_period_name** | Option<**String**> | Parameter name |  |
**parameter18_period_value** | Option<**String**> | Parameter value |  |
**parameter19_period_name** | Option<**String**> | Parameter name |  |
**parameter19_period_value** | Option<**String**> | Parameter value |  |
**parameter20_period_name** | Option<**String**> | Parameter name |  |
**parameter20_period_value** | Option<**String**> | Parameter value |  |
**parameter21_period_name** | Option<**String**> | Parameter name |  |
**parameter21_period_value** | Option<**String**> | Parameter value |  |
**parameter22_period_name** | Option<**String**> | Parameter name |  |
**parameter22_period_value** | Option<**String**> | Parameter value |  |
**parameter23_period_name** | Option<**String**> | Parameter name |  |
**parameter23_period_value** | Option<**String**> | Parameter value |  |
**parameter24_period_name** | Option<**String**> | Parameter name |  |
**parameter24_period_value** | Option<**String**> | Parameter value |  |
**parameter25_period_name** | Option<**String**> | Parameter name |  |
**parameter25_period_value** | Option<**String**> | Parameter value |  |
**parameter26_period_name** | Option<**String**> | Parameter name |  |
**parameter26_period_value** | Option<**String**> | Parameter value |  |
**parameter27_period_name** | Option<**String**> | Parameter name |  |
**parameter27_period_value** | Option<**String**> | Parameter value |  |
**parameter28_period_name** | Option<**String**> | Parameter name |  |
**parameter28_period_value** | Option<**String**> | Parameter value |  |
**parameter29_period_name** | Option<**String**> | Parameter name |  |
**parameter29_period_value** | Option<**String**> | Parameter value |  |
**parameter30_period_name** | Option<**String**> | Parameter name |  |
**parameter30_period_value** | Option<**String**> | Parameter value |  |
**parameter31_period_name** | Option<**String**> | Parameter name |  |
**parameter31_period_value** | Option<**String**> | Parameter value |  |
**parameter32_period_name** | Option<**String**> | Parameter name |  |
**parameter32_period_value** | Option<**String**> | Parameter value |  |
**parameter33_period_name** | Option<**String**> | Parameter name |  |
**parameter33_period_value** | Option<**String**> | Parameter value |  |
**parameter34_period_name** | Option<**String**> | Parameter name |  |
**parameter34_period_value** | Option<**String**> | Parameter value |  |
**parameter35_period_name** | Option<**String**> | Parameter name |  |
**parameter35_period_value** | Option<**String**> | Parameter value |  |
**parameter36_period_name** | Option<**String**> | Parameter name |  |
**parameter36_period_value** | Option<**String**> | Parameter value |  |
**parameter37_period_name** | Option<**String**> | Parameter name |  |
**parameter37_period_value** | Option<**String**> | Parameter value |  |
**parameter38_period_name** | Option<**String**> | Parameter name |  |
**parameter38_period_value** | Option<**String**> | Parameter value |  |
**parameter39_period_name** | Option<**String**> | Parameter name |  |
**parameter39_period_value** | Option<**String**> | Parameter value |  |
**parameter40_period_name** | Option<**String**> | Parameter name |  |
**parameter40_period_value** | Option<**String**> | Parameter value |  |
**parameter41_period_name** | Option<**String**> | Parameter name |  |
**parameter41_period_value** | Option<**String**> | Parameter value |  |
**parameter42_period_name** | Option<**String**> | Parameter name |  |
**parameter42_period_value** | Option<**String**> | Parameter value |  |
**parameter43_period_name** | Option<**String**> | Parameter name |  |
**parameter43_period_value** | Option<**String**> | Parameter value |  |
**parameter44_period_name** | Option<**String**> | Parameter name |  |
**parameter44_period_value** | Option<**String**> | Parameter value |  |
**parameter45_period_name** | Option<**String**> | Parameter name |  |
**parameter45_period_value** | Option<**String**> | Parameter value |  |
**parameter46_period_name** | Option<**String**> | Parameter name |  |
**parameter46_period_value** | Option<**String**> | Parameter value |  |
**parameter47_period_name** | Option<**String**> | Parameter name |  |
**parameter47_period_value** | Option<**String**> | Parameter value |  |
**parameter48_period_name** | Option<**String**> | Parameter name |  |
**parameter48_period_value** | Option<**String**> | Parameter value |  |
**parameter49_period_name** | Option<**String**> | Parameter name |  |
**parameter49_period_value** | Option<**String**> | Parameter value |  |
**parameter50_period_name** | Option<**String**> | Parameter name |  |
**parameter50_period_value** | Option<**String**> | Parameter value |  |
**parameter51_period_name** | Option<**String**> | Parameter name |  |
**parameter51_period_value** | Option<**String**> | Parameter value |  |
**parameter52_period_name** | Option<**String**> | Parameter name |  |
**parameter52_period_value** | Option<**String**> | Parameter value |  |
**parameter53_period_name** | Option<**String**> | Parameter name |  |
**parameter53_period_value** | Option<**String**> | Parameter value |  |
**parameter54_period_name** | Option<**String**> | Parameter name |  |
**parameter54_period_value** | Option<**String**> | Parameter value |  |
**parameter55_period_name** | Option<**String**> | Parameter name |  |
**parameter55_period_value** | Option<**String**> | Parameter value |  |
**parameter56_period_name** | Option<**String**> | Parameter name |  |
**parameter56_period_value** | Option<**String**> | Parameter value |  |
**parameter57_period_name** | Option<**String**> | Parameter name |  |
**parameter57_period_value** | Option<**String**> | Parameter value |  |
**parameter58_period_name** | Option<**String**> | Parameter name |  |
**parameter58_period_value** | Option<**String**> | Parameter value |  |
**parameter59_period_name** | Option<**String**> | Parameter name |  |
**parameter59_period_value** | Option<**String**> | Parameter value |  |
**parameter60_period_name** | Option<**String**> | Parameter name |  |
**parameter60_period_value** | Option<**String**> | Parameter value |  |
**parameter61_period_name** | Option<**String**> | Parameter name |  |
**parameter61_period_value** | Option<**String**> | Parameter value |  |
**parameter62_period_name** | Option<**String**> | Parameter name |  |
**parameter62_period_value** | Option<**String**> | Parameter value |  |
**parameter63_period_name** | Option<**String**> | Parameter name |  |
**parameter63_period_value** | Option<**String**> | Parameter value |  |
**parameter64_period_name** | Option<**String**> | Parameter name |  |
**parameter64_period_value** | Option<**String**> | Parameter value |  |
**parameter65_period_name** | Option<**String**> | Parameter name |  |
**parameter65_period_value** | Option<**String**> | Parameter value |  |
**parameter66_period_name** | Option<**String**> | Parameter name |  |
**parameter66_period_value** | Option<**String**> | Parameter value |  |
**parameter67_period_name** | Option<**String**> | Parameter name |  |
**parameter67_period_value** | Option<**String**> | Parameter value |  |
**parameter68_period_name** | Option<**String**> | Parameter name |  |
**parameter68_period_value** | Option<**String**> | Parameter value |  |
**parameter69_period_name** | Option<**String**> | Parameter name |  |
**parameter69_period_value** | Option<**String**> | Parameter value |  |
**parameter70_period_name** | Option<**String**> | Parameter name |  |
**parameter70_period_value** | Option<**String**> | Parameter value |  |
**parameter71_period_name** | Option<**String**> | Parameter name |  |
**parameter71_period_value** | Option<**String**> | Parameter value |  |
**parameter72_period_name** | Option<**String**> | Parameter name |  |
**parameter72_period_value** | Option<**String**> | Parameter value |  |
**parameter73_period_name** | Option<**String**> | Parameter name |  |
**parameter73_period_value** | Option<**String**> | Parameter value |  |
**parameter74_period_name** | Option<**String**> | Parameter name |  |
**parameter74_period_value** | Option<**String**> | Parameter value |  |
**parameter75_period_name** | Option<**String**> | Parameter name |  |
**parameter75_period_value** | Option<**String**> | Parameter value |  |
**parameter76_period_name** | Option<**String**> | Parameter name |  |
**parameter76_period_value** | Option<**String**> | Parameter value |  |
**parameter77_period_name** | Option<**String**> | Parameter name |  |
**parameter77_period_value** | Option<**String**> | Parameter value |  |
**parameter78_period_name** | Option<**String**> | Parameter name |  |
**parameter78_period_value** | Option<**String**> | Parameter value |  |
**parameter79_period_name** | Option<**String**> | Parameter name |  |
**parameter79_period_value** | Option<**String**> | Parameter value |  |
**parameter80_period_name** | Option<**String**> | Parameter name |  |
**parameter80_period_value** | Option<**String**> | Parameter value |  |
**parameter81_period_name** | Option<**String**> | Parameter name |  |
**parameter81_period_value** | Option<**String**> | Parameter value |  |
**parameter82_period_name** | Option<**String**> | Parameter name |  |
**parameter82_period_value** | Option<**String**> | Parameter value |  |
**parameter83_period_name** | Option<**String**> | Parameter name |  |
**parameter83_period_value** | Option<**String**> | Parameter value |  |
**parameter84_period_name** | Option<**String**> | Parameter name |  |
**parameter84_period_value** | Option<**String**> | Parameter value |  |
**parameter85_period_name** | Option<**String**> | Parameter name |  |
**parameter85_period_value** | Option<**String**> | Parameter value |  |
**parameter86_period_name** | Option<**String**> | Parameter name |  |
**parameter86_period_value** | Option<**String**> | Parameter value |  |
**parameter87_period_name** | Option<**String**> | Parameter name |  |
**parameter87_period_value** | Option<**String**> | Parameter value |  |
**parameter88_period_name** | Option<**String**> | Parameter name |  |
**parameter88_period_value** | Option<**String**> | Parameter value |  |
**parameter89_period_name** | Option<**String**> | Parameter name |  |
**parameter89_period_value** | Option<**String**> | Parameter value |  |
**parameter90_period_name** | Option<**String**> | Parameter name |  |
**parameter90_period_value** | Option<**String**> | Parameter value |  |
**parameter91_period_name** | Option<**String**> | Parameter name |  |
**parameter91_period_value** | Option<**String**> | Parameter value |  |
**parameter92_period_name** | Option<**String**> | Parameter name |  |
**parameter92_period_value** | Option<**String**> | Parameter value |  |
**parameter93_period_name** | Option<**String**> | Parameter name |  |
**parameter93_period_value** | Option<**String**> | Parameter value |  |
**parameter94_period_name** | Option<**String**> | Parameter name |  |
**parameter94_period_value** | Option<**String**> | Parameter value |  |
**parameter95_period_name** | Option<**String**> | Parameter name |  |
**parameter95_period_value** | Option<**String**> | Parameter value |  |
**parameter96_period_name** | Option<**String**> | Parameter name |  |
**parameter96_period_value** | Option<**String**> | Parameter value |  |
**parameter97_period_name** | Option<**String**> | Parameter name |  |
**parameter97_period_value** | Option<**String**> | Parameter value |  |
**parameter98_period_name** | Option<**String**> | Parameter name |  |
**parameter98_period_value** | Option<**String**> | Parameter value |  |
**parameter99_period_name** | Option<**String**> | Parameter name |  |
**parameter99_period_value** | Option<**String**> | Parameter value |  |

### Return type

[**crate::models::ApiPeriodV2010PeriodAccountPeriodCallPeriodSiprec**](api.v2010.account.call.siprec.md)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: application/x-www-form-urlencoded
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## update_siprec

> crate::models::ApiPeriodV2010PeriodAccountPeriodCallPeriodSiprec update_siprec(account_sid, call_sid, sid, status)


Stop a Siprec using either the SID of the Siprec resource or the `name` used when creating the resource

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_sid** | **String** | The SID of the [Account](https://www.twilio.com/docs/iam/api/account) that created this Siprec resource. | [required] |
**call_sid** | **String** | The SID of the [Call](https://www.twilio.com/docs/voice/api/call-resource) the Siprec resource is associated with. | [required] |
**sid** | **String** | The SID of the Siprec resource, or the `name` used when creating the resource | [required] |
**status** | **crate::models::SiprecEnumUpdateStatus** |  | [required] |

### Return type

[**crate::models::ApiPeriodV2010PeriodAccountPeriodCallPeriodSiprec**](api.v2010.account.call.siprec.md)

### Authorization

[accountSid_authToken](../README.md#accountSid_authToken)

### HTTP request headers

- **Content-Type**: application/x-www-form-urlencoded
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

