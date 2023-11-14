/// A Timestamp represents a point in time independent of any time zone or local
/// calendar, encoded as a count of seconds and fractions of seconds at
/// nanosecond resolution. The count is relative to an epoch at UTC midnight on
/// January 1, 1970, in the proleptic Gregorian calendar which extends the
/// Gregorian calendar backwards to year one.
///
/// All minutes are 60 seconds long. Leap seconds are "smeared" so that no leap
/// second table is needed for interpretation, using a [24-hour linear
/// smear](<https://developers.google.com/time/smear>).
///
/// The range is from 0001-01-01T00:00:00Z to 9999-12-31T23:59:59.999999999Z. By
/// restricting to that range, we ensure that we can convert to and from [RFC
/// 3339](<https://www.ietf.org/rfc/rfc3339.txt>) date strings.
///
/// # Examples
///
/// Example 1: Compute Timestamp from POSIX `time()`.
///
/// ```text
/// Timestamp timestamp;
/// timestamp.set_seconds(time(NULL));
/// timestamp.set_nanos(0);
/// ```
///
/// Example 2: Compute Timestamp from POSIX `gettimeofday()`.
///
/// ```text
/// struct timeval tv;
/// gettimeofday(&tv, NULL);
///
/// Timestamp timestamp;
/// timestamp.set_seconds(tv.tv_sec);
/// timestamp.set_nanos(tv.tv_usec * 1000);
/// ```
///
/// Example 3: Compute Timestamp from Win32 `GetSystemTimeAsFileTime()`.
///
/// ```text
/// FILETIME ft;
/// GetSystemTimeAsFileTime(&ft);
/// UINT64 ticks = (((UINT64)ft.dwHighDateTime) << 32) | ft.dwLowDateTime;
///
/// // A Windows tick is 100 nanoseconds. Windows epoch 1601-01-01T00:00:00Z
/// // is 11644473600 seconds before Unix epoch 1970-01-01T00:00:00Z.
/// Timestamp timestamp;
/// timestamp.set_seconds((INT64) ((ticks / 10000000) - 11644473600LL));
/// timestamp.set_nanos((INT32) ((ticks % 10000000) * 100));
/// ```
///
/// Example 4: Compute Timestamp from Java `System.currentTimeMillis()`.
///
/// ```text
/// long millis = System.currentTimeMillis();
///
/// Timestamp timestamp = Timestamp.newBuilder().setSeconds(millis / 1000)
///      .setNanos((int) ((millis % 1000) * 1000000)).build();
/// ```
///
/// Example 5: Compute Timestamp from Java `Instant.now()`.
///
/// ```text
/// Instant now = Instant.now();
///
/// Timestamp timestamp =
///      Timestamp.newBuilder().setSeconds(now.getEpochSecond())
///          .setNanos(now.getNano()).build();
/// ```
///
///
/// Example 6: Compute Timestamp from current time in Python.
///
/// ```text
/// timestamp = Timestamp()
/// timestamp.GetCurrentTime()
/// ```
///
/// # JSON Mapping
///
/// In JSON format, the Timestamp type is encoded as a string in the
/// [RFC 3339](<https://www.ietf.org/rfc/rfc3339.txt>) format. That is, the
/// format is "{year}-{month}-{day}T{hour}:{min}:{sec}\[.{frac_sec}\]Z"
/// where {year} is always expressed using four digits while {month}, {day},
/// {hour}, {min}, and {sec} are zero-padded to two digits each. The fractional
/// seconds, which can go up to 9 digits (i.e. up to 1 nanosecond resolution),
/// are optional. The "Z" suffix indicates the timezone ("UTC"); the timezone
/// is required. A proto3 JSON serializer should always use UTC (as indicated by
/// "Z") when printing the Timestamp type and a proto3 JSON parser should be
/// able to accept both UTC and other timezones (as indicated by an offset).
///
/// For example, "2017-01-15T01:30:15.01Z" encodes 15.01 seconds past
/// 01:30 UTC on January 15, 2017.
///
/// In JavaScript, one can convert a Date object to this format using the
/// standard
/// [toISOString()](<https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Date/toISOString>)
/// method. In Python, a standard `datetime.datetime` object can be converted
/// to this format using
/// [`strftime`](<https://docs.python.org/2/library/time.html#time.strftime>) with
/// the time format spec '%Y-%m-%dT%H:%M:%S.%fZ'. Likewise, in Java, one can use
/// the Joda Time's [`ISODateTimeFormat.dateTime()`](
/// <http://www.joda.org/joda-time/apidocs/org/joda/time/format/ISODateTimeFormat.html#dateTime%2D%2D>
/// ) to obtain a formatter capable of generating timestamps in this format.
///
///
#[derive(Copy)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Timestamp {
    /// Represents seconds of UTC time since Unix epoch
    /// 1970-01-01T00:00:00Z. Must be from 0001-01-01T00:00:00Z to
    /// 9999-12-31T23:59:59Z inclusive.
    #[prost(int64, tag = "1")]
    pub seconds: i64,
    /// Non-negative fractions of a second at nanosecond resolution. Negative
    /// second values with fractions must still have non-negative nanos values
    /// that count forward in time. Must be from 0 to 999,999,999
    /// inclusive.
    #[prost(int32, tag = "2")]
    pub nanos: i32,
}
/// Wrapper message for `double`.
///
/// The JSON representation for `DoubleValue` is JSON number.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DoubleValue {
    /// The double value.
    #[prost(double, tag = "1")]
    pub value: f64,
}
/// Wrapper message for `float`.
///
/// The JSON representation for `FloatValue` is JSON number.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FloatValue {
    /// The float value.
    #[prost(float, tag = "1")]
    pub value: f32,
}
/// Wrapper message for `int64`.
///
/// The JSON representation for `Int64Value` is JSON string.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Int64Value {
    /// The int64 value.
    #[prost(int64, tag = "1")]
    pub value: i64,
}
/// Wrapper message for `uint64`.
///
/// The JSON representation for `UInt64Value` is JSON string.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UInt64Value {
    /// The uint64 value.
    #[prost(uint64, tag = "1")]
    pub value: u64,
}
/// Wrapper message for `int32`.
///
/// The JSON representation for `Int32Value` is JSON number.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Int32Value {
    /// The int32 value.
    #[prost(int32, tag = "1")]
    pub value: i32,
}
/// Wrapper message for `uint32`.
///
/// The JSON representation for `UInt32Value` is JSON number.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UInt32Value {
    /// The uint32 value.
    #[prost(uint32, tag = "1")]
    pub value: u32,
}
/// Wrapper message for `bool`.
///
/// The JSON representation for `BoolValue` is JSON `true` and `false`.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BoolValue {
    /// The bool value.
    #[prost(bool, tag = "1")]
    pub value: bool,
}
/// Wrapper message for `string`.
///
/// The JSON representation for `StringValue` is JSON string.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct StringValue {
    /// The string value.
    #[prost(string, tag = "1")]
    pub value: ::prost::alloc::string::String,
}
/// Wrapper message for `bytes`.
///
/// The JSON representation for `BytesValue` is JSON string.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BytesValue {
    /// The bytes value.
    #[prost(bytes = "vec", tag = "1")]
    pub value: ::prost::alloc::vec::Vec<u8>,
}
/// `Any` contains an arbitrary serialized protocol buffer message along with a
/// URL that describes the type of the serialized message.
///
/// Protobuf library provides support to pack/unpack Any values in the form
/// of utility functions or additional generated methods of the Any type.
///
/// Example 1: Pack and unpack a message in C++.
///
/// ```text
/// Foo foo = ...;
/// Any any;
/// any.PackFrom(foo);
/// ...
/// if (any.UnpackTo(&foo)) {
///    ...
/// }
/// ```
///
/// Example 2: Pack and unpack a message in Java.
///
/// ```text
/// Foo foo = ...;
/// Any any = Any.pack(foo);
/// ...
/// if (any.is(Foo.class)) {
///    foo = any.unpack(Foo.class);
/// }
/// ```
///
///   Example 3: Pack and unpack a message in Python.
///
/// ```text
/// foo = Foo(...)
/// any = Any()
/// any.Pack(foo)
/// ...
/// if any.Is(Foo.DESCRIPTOR):
///    any.Unpack(foo)
///    ...
/// ```
///
///   Example 4: Pack and unpack a message in Go
///
/// ```text
///   foo := &pb.Foo{...}
///   any, err := anypb.New(foo)
///   if err != nil {
///     ...
///   }
///   ...
///   foo := &pb.Foo{}
///   if err := any.UnmarshalTo(foo); err != nil {
///     ...
///   }
/// ```
///
/// The pack methods provided by protobuf library will by default use
/// 'type.googleapis.com/full.type.name' as the type URL and the unpack
/// methods only use the fully qualified type name after the last '/'
/// in the type URL, for example "foo.bar.com/x/y.z" will yield type
/// name "y.z".
///
///
/// JSON
/// ====
/// The JSON representation of an `Any` value uses the regular
/// representation of the deserialized, embedded message, with an
/// additional field `@type` which contains the type URL. Example:
///
/// ```text
/// package google.profile;
/// message Person {
///    string first_name = 1;
///    string last_name = 2;
/// }
///
/// {
///    "@type": "type.googleapis.com/google.profile.Person",
///    "firstName": <string>,
///    "lastName": <string>
/// }
/// ```
///
/// If the embedded message type is well-known and has a custom JSON
/// representation, that representation will be embedded adding a field
/// `value` which holds the custom JSON in addition to the `@type`
/// field. Example (for message [google.protobuf.Duration][]):
///
/// ```text
/// {
///    "@type": "type.googleapis.com/google.protobuf.Duration",
///    "value": "1.212s"
/// }
/// ```
///
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Any {
    /// A URL/resource name that uniquely identifies the type of the serialized
    /// protocol buffer message. This string must contain at least
    /// one "/" character. The last segment of the URL's path must represent
    /// the fully qualified name of the type (as in
    /// `path/google.protobuf.Duration`). The name should be in a canonical form
    /// (e.g., leading "." is not accepted).
    ///
    /// In practice, teams usually precompile into the binary all types that they
    /// expect it to use in the context of Any. However, for URLs which use the
    /// scheme `http`, `https`, or no scheme, one can optionally set up a type
    /// server that maps type URLs to message definitions as follows:
    ///
    /// * If no scheme is provided, `https` is assumed.
    /// * An HTTP GET on the URL must yield a [google.protobuf.Type][]
    ///    value in binary format, or produce an error.
    /// * Applications are allowed to cache lookup results based on the
    ///    URL, or have them precompiled into a binary to avoid any
    ///    lookup. Therefore, binary compatibility needs to be preserved
    ///    on changes to types. (Use versioned type names to manage
    ///    breaking changes.)
    ///
    /// Note: this functionality is not currently available in the official
    /// protobuf release, and it is not used for type URLs beginning with
    /// type.googleapis.com.
    ///
    /// Schemes other than `http`, `https` (or the empty scheme) might be
    /// used with implementation specific semantics.
    ///
    #[prost(string, tag = "1")]
    pub type_url: ::prost::alloc::string::String,
    /// Must be a valid serialized protocol buffer of the above specified type.
    #[prost(bytes = "vec", tag = "2")]
    pub value: ::prost::alloc::vec::Vec<u8>,
}
/// `FieldMask` represents a set of symbolic field paths, for example:
///
/// ```text
/// paths: "f.a"
/// paths: "f.b.d"
/// ```
///
/// Here `f` represents a field in some root message, `a` and `b`
/// fields in the message found in `f`, and `d` a field found in the
/// message in `f.b`.
///
/// Field masks are used to specify a subset of fields that should be
/// returned by a get operation or modified by an update operation.
/// Field masks also have a custom JSON encoding (see below).
///
/// # Field Masks in Projections
///
/// When used in the context of a projection, a response message or
/// sub-message is filtered by the API to only contain those fields as
/// specified in the mask. For example, if the mask in the previous
/// example is applied to a response message as follows:
///
/// ```text
/// f {
///    a : 22
///    b {
///      d : 1
///      x : 2
///    }
///    y : 13
/// }
/// z: 8
/// ```
///
/// The result will not contain specific values for fields x,y and z
/// (their value will be set to the default, and omitted in proto text
/// output):
///
///
/// ```text
/// f {
///    a : 22
///    b {
///      d : 1
///    }
/// }
/// ```
///
/// A repeated field is not allowed except at the last position of a
/// paths string.
///
/// If a FieldMask object is not present in a get operation, the
/// operation applies to all fields (as if a FieldMask of all fields
/// had been specified).
///
/// Note that a field mask does not necessarily apply to the
/// top-level response message. In case of a REST get operation, the
/// field mask applies directly to the response, but in case of a REST
/// list operation, the mask instead applies to each individual message
/// in the returned resource list. In case of a REST custom method,
/// other definitions may be used. Where the mask applies will be
/// clearly documented together with its declaration in the API.  In
/// any case, the effect on the returned resource/resources is required
/// behavior for APIs.
///
/// # Field Masks in Update Operations
///
/// A field mask in update operations specifies which fields of the
/// targeted resource are going to be updated. The API is required
/// to only change the values of the fields as specified in the mask
/// and leave the others untouched. If a resource is passed in to
/// describe the updated values, the API ignores the values of all
/// fields not covered by the mask.
///
/// If a repeated field is specified for an update operation, new values will
/// be appended to the existing repeated field in the target resource. Note that
/// a repeated field is only allowed in the last position of a `paths` string.
///
/// If a sub-message is specified in the last position of the field mask for an
/// update operation, then new value will be merged into the existing sub-message
/// in the target resource.
///
/// For example, given the target message:
///
/// ```text
/// f {
///    b {
///      d: 1
///      x: 2
///    }
///    c: \[1\]
/// }
/// ```
///
/// And an update message:
///
/// ```text
/// f {
///    b {
///      d: 10
///    }
///    c: \[2\]
/// }
/// ```
///
/// then if the field mask is:
///
/// ```text
///   paths: \["f.b", "f.c"\]
/// ```
///
/// then the result will be:
///
/// ```text
/// f {
///    b {
///      d: 10
///      x: 2
///    }
///    c: \[1, 2\]
/// }
/// ```
///
/// An implementation may provide options to override this default behavior for
/// repeated and message fields.
///
/// In order to reset a field's value to the default, the field must
/// be in the mask and set to the default value in the provided resource.
/// Hence, in order to reset all fields of a resource, provide a default
/// instance of the resource and set all fields in the mask, or do
/// not provide a mask as described below.
///
/// If a field mask is not present on update, the operation applies to
/// all fields (as if a field mask of all fields has been specified).
/// Note that in the presence of schema evolution, this may mean that
/// fields the client does not know and has therefore not filled into
/// the request will be reset to their default. If this is unwanted
/// behavior, a specific service may require a client to always specify
/// a field mask, producing an error if not.
///
/// As with get operations, the location of the resource which
/// describes the updated values in the request message depends on the
/// operation kind. In any case, the effect of the field mask is
/// required to be honored by the API.
///
/// ## Considerations for HTTP REST
///
/// The HTTP kind of an update operation which uses a field mask must
/// be set to PATCH instead of PUT in order to satisfy HTTP semantics
/// (PUT must only be used for full updates).
///
/// # JSON Encoding of Field Masks
///
/// In JSON, a field mask is encoded as a single string where paths are
/// separated by a comma. Fields name in each path are converted
/// to/from lower-camel naming conventions.
///
/// As an example, consider the following message declarations:
///
/// ```text
/// message Profile {
///    User user = 1;
///    Photo photo = 2;
/// }
/// message User {
///    string display_name = 1;
///    string address = 2;
/// }
/// ```
///
/// In proto a field mask for `Profile` may look as such:
///
/// ```text
/// mask {
///    paths: "user.display_name"
///    paths: "photo"
/// }
/// ```
///
/// In JSON, the same mask is represented as below:
///
/// ```text
/// {
///    mask: "user.displayName,photo"
/// }
/// ```
///
/// # Field Masks and Oneof Fields
///
/// Field masks treat fields in oneofs just as regular fields. Consider the
/// following message:
///
/// ```text
/// message SampleMessage {
///    oneof test_oneof {
///      string name = 4;
///      SubMessage sub_message = 9;
///    }
/// }
/// ```
///
/// The field mask can be:
///
/// ```text
/// mask {
///    paths: "name"
/// }
/// ```
///
/// Or:
///
/// ```text
/// mask {
///    paths: "sub_message"
/// }
/// ```
///
/// Note that oneof type names ("test_oneof" in this case) cannot be used in
/// paths.
///
/// ## Field Mask Verification
///
/// The implementation of any API method which has a FieldMask type field in the
/// request should verify the included field paths, and return an
/// `INVALID_ARGUMENT` error if any path is unmappable.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FieldMask {
    /// The set of field mask paths.
    #[prost(string, repeated, tag = "1")]
    pub paths: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
/// A generic empty message that you can re-use to avoid defining duplicated
/// empty messages in your APIs. A typical example is to use it as the request
/// or the response type of an API method. For instance:
///
/// ```text
/// service Foo {
///    rpc Bar(google.protobuf.Empty) returns (google.protobuf.Empty);
/// }
/// ```
///
/// The JSON representation for `Empty` is empty JSON object `{}`.
#[derive(Copy)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Empty {}
/// A Duration represents a signed, fixed-length span of time represented
/// as a count of seconds and fractions of seconds at nanosecond
/// resolution. It is independent of any calendar and concepts like "day"
/// or "month". It is related to Timestamp in that the difference between
/// two Timestamp values is a Duration and it can be added or subtracted
/// from a Timestamp. Range is approximately +-10,000 years.
///
/// # Examples
///
/// Example 1: Compute Duration from two Timestamps in pseudo code.
///
/// ```text
/// Timestamp start = ...;
/// Timestamp end = ...;
/// Duration duration = ...;
///
/// duration.seconds = end.seconds - start.seconds;
/// duration.nanos = end.nanos - start.nanos;
///
/// if (duration.seconds < 0 && duration.nanos > 0) {
///    duration.seconds += 1;
///    duration.nanos -= 1000000000;
/// } else if (duration.seconds > 0 && duration.nanos < 0) {
///    duration.seconds -= 1;
///    duration.nanos += 1000000000;
/// }
/// ```
///
/// Example 2: Compute Timestamp from Timestamp + Duration in pseudo code.
///
/// ```text
/// Timestamp start = ...;
/// Duration duration = ...;
/// Timestamp end = ...;
///
/// end.seconds = start.seconds + duration.seconds;
/// end.nanos = start.nanos + duration.nanos;
///
/// if (end.nanos < 0) {
///    end.seconds -= 1;
///    end.nanos += 1000000000;
/// } else if (end.nanos >= 1000000000) {
///    end.seconds += 1;
///    end.nanos -= 1000000000;
/// }
/// ```
///
/// Example 3: Compute Duration from datetime.timedelta in Python.
///
/// ```text
/// td = datetime.timedelta(days=3, minutes=10)
/// duration = Duration()
/// duration.FromTimedelta(td)
/// ```
///
/// # JSON Mapping
///
/// In JSON format, the Duration type is encoded as a string rather than an
/// object, where the string ends in the suffix "s" (indicating seconds) and
/// is preceded by the number of seconds, with nanoseconds expressed as
/// fractional seconds. For example, 3 seconds with 0 nanoseconds should be
/// encoded in JSON format as "3s", while 3 seconds and 1 nanosecond should
/// be expressed in JSON format as "3.000000001s", and 3 seconds and 1
/// microsecond should be expressed in JSON format as "3.000001s".
///
///
#[derive(Copy)]
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Duration {
    /// Signed seconds of the span of time. Must be from -315,576,000,000
    /// to +315,576,000,000 inclusive. Note: these bounds are computed from:
    /// 60 sec/min * 60 min/hr * 24 hr/day * 365.25 days/year * 10000 years
    #[prost(int64, tag = "1")]
    pub seconds: i64,
    /// Signed fractions of a second at nanosecond resolution of the span
    /// of time. Durations less than one second are represented with a 0
    /// `seconds` field and a positive or negative `nanos` field. For durations
    /// of one second or more, a non-zero value for the `nanos` field must be
    /// of the same sign as the `seconds` field. Must be from -999,999,999
    /// to +999,999,999 inclusive.
    #[prost(int32, tag = "2")]
    pub nanos: i32,
}
