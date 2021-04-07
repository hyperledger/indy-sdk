package org.hyperledger.indy.sdk;

import com.sun.jna.FromNativeContext;
import com.sun.jna.ToNativeContext;
import com.sun.jna.TypeConverter;

public final class IndyBool {
	static final TypeConverter MAPPER = new TypeConverter() {
		@Override
		public Class<?> nativeType() {
			return Byte.class;
		}

		@Override
		public Object toNative(Object value, ToNativeContext context) {
			if(value == null) {
				return IndyBool.FALSE.toByte();
			}
			else {
				final IndyBool bool = (IndyBool) value;
				return bool.toByte();
			}
		}

		@Override
		public Object fromNative(Object nativeValue, FromNativeContext context) {
			if(nativeValue == null) {
				return IndyBool.FALSE;
			}
			else {
				final byte value = (byte) nativeValue;
				return value == 1 ? IndyBool.TRUE : IndyBool.FALSE;
			}
		}
	};

	private static final IndyBool TRUE = new IndyBool(true);
	private static final IndyBool FALSE = new IndyBool(false);

	private final boolean value;

	private IndyBool(boolean value) {
		this.value = value;
	}

	public boolean value() {
		return value;
	}

	private byte toByte() {
		return (byte) (value ? 1 : 0);
	}
}
