# ApiPeriodV2010PeriodAccountPeriodUsagePeriodUsageTrigger

## Properties

Name | Type | Description | Notes
------------ | ------------- | ------------- | -------------
**account_sid** | Option<**String**> | The SID of the [Account](https://www.twilio.com/docs/iam/api/account) that the trigger monitors. | [optional]
**api_version** | Option<**String**> | The API version used to create the resource. | [optional]
**callback_method** | Option<**String**> | The HTTP method we use to call `callback_url`. Can be: `GET` or `POST`. | [optional]
**callback_url** | Option<**String**> | The URL we call using the `callback_method` when the trigger fires. | [optional]
**current_value** | Option<**String**> | The current value of the field the trigger is watching. | [optional]
**date_created** | Option<**String**> | The date and time in GMT that the resource was created specified in [RFC 2822](https://www.ietf.org/rfc/rfc2822.txt) format. | [optional]
**date_fired** | Option<**String**> | The date and time in GMT that the trigger was last fired specified in [RFC 2822](https://www.ietf.org/rfc/rfc2822.txt) format. | [optional]
**date_updated** | Option<**String**> | The date and time in GMT that the resource was last updated specified in [RFC 2822](https://www.ietf.org/rfc/rfc2822.txt) format. | [optional]
**friendly_name** | Option<**String**> | The string that you assigned to describe the trigger. | [optional]
**recurring** | Option<[**crate::models::UsageTriggerEnumRecurring**](usage_trigger_enum_recurring.md)> |  | [optional]
**sid** | Option<**String**> | The unique string that that we created to identify the UsageTrigger resource. | [optional]
**trigger_by** | Option<[**crate::models::UsageTriggerEnumTriggerField**](usage_trigger_enum_trigger_field.md)> |  | [optional]
**trigger_value** | Option<**String**> | The value at which the trigger will fire.  Must be a positive, numeric value. | [optional]
**uri** | Option<**String**> | The URI of the resource, relative to `https://api.twilio.com`. | [optional]
**usage_category** | Option<[**crate::models::UsageTriggerEnumUsageCategory**](usage_trigger_enum_usage_category.md)> |  | [optional]
**usage_record_uri** | Option<**String**> | The URI of the [UsageRecord](https://www.twilio.com/docs/usage/api/usage-record) resource this trigger watches, relative to `https://api.twilio.com`. | [optional]

[[Back to Model list]](../README.md#documentation-for-models) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to README]](../README.md)


