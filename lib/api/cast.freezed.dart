// GENERATED CODE - DO NOT MODIFY BY HAND
// coverage:ignore-file
// ignore_for_file: type=lint
// ignore_for_file: unused_element, deprecated_member_use, deprecated_member_use_from_same_package, use_function_type_syntax_for_parameters, unnecessary_const, avoid_init_to_null, invalid_override_different_default_values_named, prefer_expression_function_bodies, annotate_overrides, invalid_annotation_target, unnecessary_question_mark

part of 'cast.dart';

// **************************************************************************
// FreezedGenerator
// **************************************************************************

// dart format off
T _$identity<T>(T value) => value;

/// @nodoc
mixin _$ProjectorInfo {
  String get friendlyName;
  String get ip;
  String get locationXmlUrl;
  String? get avTransportUrl;
  String? get renderingControlUrl;

  /// Create a copy of ProjectorInfo
  /// with the given fields replaced by the non-null parameter values.
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  $ProjectorInfoCopyWith<ProjectorInfo> get copyWith =>
      _$ProjectorInfoCopyWithImpl<ProjectorInfo>(
          this as ProjectorInfo, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is ProjectorInfo &&
            (identical(other.friendlyName, friendlyName) ||
                other.friendlyName == friendlyName) &&
            (identical(other.ip, ip) || other.ip == ip) &&
            (identical(other.locationXmlUrl, locationXmlUrl) ||
                other.locationXmlUrl == locationXmlUrl) &&
            (identical(other.avTransportUrl, avTransportUrl) ||
                other.avTransportUrl == avTransportUrl) &&
            (identical(other.renderingControlUrl, renderingControlUrl) ||
                other.renderingControlUrl == renderingControlUrl));
  }

  @override
  int get hashCode => Object.hash(runtimeType, friendlyName, ip, locationXmlUrl,
      avTransportUrl, renderingControlUrl);

  @override
  String toString() {
    return 'ProjectorInfo(friendlyName: $friendlyName, ip: $ip, locationXmlUrl: $locationXmlUrl, avTransportUrl: $avTransportUrl, renderingControlUrl: $renderingControlUrl)';
  }
}

/// @nodoc
abstract mixin class $ProjectorInfoCopyWith<$Res> {
  factory $ProjectorInfoCopyWith(
          ProjectorInfo value, $Res Function(ProjectorInfo) _then) =
      _$ProjectorInfoCopyWithImpl;
  @useResult
  $Res call(
      {String friendlyName,
      String ip,
      String locationXmlUrl,
      String? avTransportUrl,
      String? renderingControlUrl});
}

/// @nodoc
class _$ProjectorInfoCopyWithImpl<$Res>
    implements $ProjectorInfoCopyWith<$Res> {
  _$ProjectorInfoCopyWithImpl(this._self, this._then);

  final ProjectorInfo _self;
  final $Res Function(ProjectorInfo) _then;

  /// Create a copy of ProjectorInfo
  /// with the given fields replaced by the non-null parameter values.
  @pragma('vm:prefer-inline')
  @override
  $Res call({
    Object? friendlyName = null,
    Object? ip = null,
    Object? locationXmlUrl = null,
    Object? avTransportUrl = freezed,
    Object? renderingControlUrl = freezed,
  }) {
    return _then(_self.copyWith(
      friendlyName: null == friendlyName
          ? _self.friendlyName
          : friendlyName // ignore: cast_nullable_to_non_nullable
              as String,
      ip: null == ip
          ? _self.ip
          : ip // ignore: cast_nullable_to_non_nullable
              as String,
      locationXmlUrl: null == locationXmlUrl
          ? _self.locationXmlUrl
          : locationXmlUrl // ignore: cast_nullable_to_non_nullable
              as String,
      avTransportUrl: freezed == avTransportUrl
          ? _self.avTransportUrl
          : avTransportUrl // ignore: cast_nullable_to_non_nullable
              as String?,
      renderingControlUrl: freezed == renderingControlUrl
          ? _self.renderingControlUrl
          : renderingControlUrl // ignore: cast_nullable_to_non_nullable
              as String?,
    ));
  }
}

/// Adds pattern-matching-related methods to [ProjectorInfo].
extension ProjectorInfoPatterns on ProjectorInfo {
  /// A variant of `map` that fallback to returning `orElse`.
  ///
  /// It is equivalent to doing:
  /// ```dart
  /// switch (sealedClass) {
  ///   case final Subclass value:
  ///     return ...;
  ///   case _:
  ///     return orElse();
  /// }
  /// ```

  @optionalTypeArgs
  TResult maybeMap<TResult extends Object?>(
    TResult Function(_ProjectorInfo value)? $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _ProjectorInfo() when $default != null:
        return $default(_that);
      case _:
        return orElse();
    }
  }

  /// A `switch`-like method, using callbacks.
  ///
  /// Callbacks receives the raw object, upcasted.
  /// It is equivalent to doing:
  /// ```dart
  /// switch (sealedClass) {
  ///   case final Subclass value:
  ///     return ...;
  ///   case final Subclass2 value:
  ///     return ...;
  /// }
  /// ```

  @optionalTypeArgs
  TResult map<TResult extends Object?>(
    TResult Function(_ProjectorInfo value) $default,
  ) {
    final _that = this;
    switch (_that) {
      case _ProjectorInfo():
        return $default(_that);
    }
  }

  /// A variant of `map` that fallback to returning `null`.
  ///
  /// It is equivalent to doing:
  /// ```dart
  /// switch (sealedClass) {
  ///   case final Subclass value:
  ///     return ...;
  ///   case _:
  ///     return null;
  /// }
  /// ```

  @optionalTypeArgs
  TResult? mapOrNull<TResult extends Object?>(
    TResult? Function(_ProjectorInfo value)? $default,
  ) {
    final _that = this;
    switch (_that) {
      case _ProjectorInfo() when $default != null:
        return $default(_that);
      case _:
        return null;
    }
  }

  /// A variant of `when` that fallback to an `orElse` callback.
  ///
  /// It is equivalent to doing:
  /// ```dart
  /// switch (sealedClass) {
  ///   case Subclass(:final field):
  ///     return ...;
  ///   case _:
  ///     return orElse();
  /// }
  /// ```

  @optionalTypeArgs
  TResult maybeWhen<TResult extends Object?>(
    TResult Function(String friendlyName, String ip, String locationXmlUrl,
            String? avTransportUrl, String? renderingControlUrl)?
        $default, {
    required TResult orElse(),
  }) {
    final _that = this;
    switch (_that) {
      case _ProjectorInfo() when $default != null:
        return $default(_that.friendlyName, _that.ip, _that.locationXmlUrl,
            _that.avTransportUrl, _that.renderingControlUrl);
      case _:
        return orElse();
    }
  }

  /// A `switch`-like method, using callbacks.
  ///
  /// As opposed to `map`, this offers destructuring.
  /// It is equivalent to doing:
  /// ```dart
  /// switch (sealedClass) {
  ///   case Subclass(:final field):
  ///     return ...;
  ///   case Subclass2(:final field2):
  ///     return ...;
  /// }
  /// ```

  @optionalTypeArgs
  TResult when<TResult extends Object?>(
    TResult Function(String friendlyName, String ip, String locationXmlUrl,
            String? avTransportUrl, String? renderingControlUrl)
        $default,
  ) {
    final _that = this;
    switch (_that) {
      case _ProjectorInfo():
        return $default(_that.friendlyName, _that.ip, _that.locationXmlUrl,
            _that.avTransportUrl, _that.renderingControlUrl);
    }
  }

  /// A variant of `when` that fallback to returning `null`
  ///
  /// It is equivalent to doing:
  /// ```dart
  /// switch (sealedClass) {
  ///   case Subclass(:final field):
  ///     return ...;
  ///   case _:
  ///     return null;
  /// }
  /// ```

  @optionalTypeArgs
  TResult? whenOrNull<TResult extends Object?>(
    TResult? Function(String friendlyName, String ip, String locationXmlUrl,
            String? avTransportUrl, String? renderingControlUrl)?
        $default,
  ) {
    final _that = this;
    switch (_that) {
      case _ProjectorInfo() when $default != null:
        return $default(_that.friendlyName, _that.ip, _that.locationXmlUrl,
            _that.avTransportUrl, _that.renderingControlUrl);
      case _:
        return null;
    }
  }
}

/// @nodoc

class _ProjectorInfo extends ProjectorInfo {
  const _ProjectorInfo(
      {required this.friendlyName,
      required this.ip,
      required this.locationXmlUrl,
      this.avTransportUrl,
      this.renderingControlUrl})
      : super._();

  @override
  final String friendlyName;
  @override
  final String ip;
  @override
  final String locationXmlUrl;
  @override
  final String? avTransportUrl;
  @override
  final String? renderingControlUrl;

  /// Create a copy of ProjectorInfo
  /// with the given fields replaced by the non-null parameter values.
  @override
  @JsonKey(includeFromJson: false, includeToJson: false)
  @pragma('vm:prefer-inline')
  _$ProjectorInfoCopyWith<_ProjectorInfo> get copyWith =>
      __$ProjectorInfoCopyWithImpl<_ProjectorInfo>(this, _$identity);

  @override
  bool operator ==(Object other) {
    return identical(this, other) ||
        (other.runtimeType == runtimeType &&
            other is _ProjectorInfo &&
            (identical(other.friendlyName, friendlyName) ||
                other.friendlyName == friendlyName) &&
            (identical(other.ip, ip) || other.ip == ip) &&
            (identical(other.locationXmlUrl, locationXmlUrl) ||
                other.locationXmlUrl == locationXmlUrl) &&
            (identical(other.avTransportUrl, avTransportUrl) ||
                other.avTransportUrl == avTransportUrl) &&
            (identical(other.renderingControlUrl, renderingControlUrl) ||
                other.renderingControlUrl == renderingControlUrl));
  }

  @override
  int get hashCode => Object.hash(runtimeType, friendlyName, ip, locationXmlUrl,
      avTransportUrl, renderingControlUrl);

  @override
  String toString() {
    return 'ProjectorInfo(friendlyName: $friendlyName, ip: $ip, locationXmlUrl: $locationXmlUrl, avTransportUrl: $avTransportUrl, renderingControlUrl: $renderingControlUrl)';
  }
}

/// @nodoc
abstract mixin class _$ProjectorInfoCopyWith<$Res>
    implements $ProjectorInfoCopyWith<$Res> {
  factory _$ProjectorInfoCopyWith(
          _ProjectorInfo value, $Res Function(_ProjectorInfo) _then) =
      __$ProjectorInfoCopyWithImpl;
  @override
  @useResult
  $Res call(
      {String friendlyName,
      String ip,
      String locationXmlUrl,
      String? avTransportUrl,
      String? renderingControlUrl});
}

/// @nodoc
class __$ProjectorInfoCopyWithImpl<$Res>
    implements _$ProjectorInfoCopyWith<$Res> {
  __$ProjectorInfoCopyWithImpl(this._self, this._then);

  final _ProjectorInfo _self;
  final $Res Function(_ProjectorInfo) _then;

  /// Create a copy of ProjectorInfo
  /// with the given fields replaced by the non-null parameter values.
  @override
  @pragma('vm:prefer-inline')
  $Res call({
    Object? friendlyName = null,
    Object? ip = null,
    Object? locationXmlUrl = null,
    Object? avTransportUrl = freezed,
    Object? renderingControlUrl = freezed,
  }) {
    return _then(_ProjectorInfo(
      friendlyName: null == friendlyName
          ? _self.friendlyName
          : friendlyName // ignore: cast_nullable_to_non_nullable
              as String,
      ip: null == ip
          ? _self.ip
          : ip // ignore: cast_nullable_to_non_nullable
              as String,
      locationXmlUrl: null == locationXmlUrl
          ? _self.locationXmlUrl
          : locationXmlUrl // ignore: cast_nullable_to_non_nullable
              as String,
      avTransportUrl: freezed == avTransportUrl
          ? _self.avTransportUrl
          : avTransportUrl // ignore: cast_nullable_to_non_nullable
              as String?,
      renderingControlUrl: freezed == renderingControlUrl
          ? _self.renderingControlUrl
          : renderingControlUrl // ignore: cast_nullable_to_non_nullable
              as String?,
    ));
  }
}

// dart format on
