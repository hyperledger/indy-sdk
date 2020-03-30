package com.evernym.sdk.vcx;


import org.apache.commons.lang3.builder.EqualsBuilder;
import org.apache.commons.lang3.builder.HashCodeBuilder;
import org.apache.commons.lang3.builder.ToStringBuilder;
import org.apache.commons.lang3.builder.ToStringStyle;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.util.HashMap;
import java.util.Iterator;
import java.util.Map;
import java.util.concurrent.ConcurrentHashMap;
import java.util.concurrent.atomic.AtomicInteger;

import java.util.concurrent.CompletableFuture;

/**
 * Common functionality for the APIs, JSON parameters, and results used
 * by the Java wrapper of libvcx.
 */
public class VcxJava {

	private static final Logger logger = LoggerFactory.getLogger("VcxJava");
	/*
	 * API
	 */

	/**
	 * Common features for API classes.
	 */
	public static class API {

		/*
		 * FUTURES
		 */

		private static AtomicInteger atomicInteger = new AtomicInteger();
		private static Map<Integer, CompletableFuture<?>> futures = new ConcurrentHashMap<Integer, CompletableFuture<?>>();

		/**
		 * Generates and returns a new command handle.
		 * 
		 * @return The new command handle.
		 */
		static int newCommandHandle() {

			return atomicInteger.incrementAndGet();
		}

		/**
		 * Adds a future to track.
		 * 
		 * @param future The future to track.
		 * @return The command handle the future is being tracked against.
		 */
		protected static int addFuture(CompletableFuture<?> future) {

			int commandHandle = newCommandHandle();
			assert(! futures.containsKey(commandHandle));
			futures.put(commandHandle, future);
			logger.debug("added future with command handle: {}", commandHandle);
			return commandHandle;
		}

		/**
		 * Stops tracking the future associated with the provided command handle and returns it.
		 * 
		 * @param commandHandle The command handle for the future to stop tracking.
		 * @return The future associated with the command handle.
		 */
		protected static CompletableFuture<?> removeFuture(int commandHandle) {
			logger.debug("removeFuture: callback completed for command handle: {}", commandHandle);
			CompletableFuture<?> future = futures.remove(commandHandle);
			assert(future != null);

			return future;
		}

		/*
		 * ERROR CHECKING
		 */

		/**
		 * Sets the provided future with an exception if the error code provided does not indicate success.
		 * 
		 * @param future The future.
		 * @param err The error value to check.
		 * @return true if the error code indicate SUCCESS, otherwise false.
		 */
		protected static boolean checkCallback(CompletableFuture<?> future, int err) {
			ErrorCode errorCode = ErrorCode.UNKNOWN_ERROR;

			try {
				errorCode = ErrorCode.valueOf(err);
				if (errorCode == null) {
					errorCode = ErrorCode.UNKNOWN_ERROR;
				}
			} catch(Exception e) {
				logger.warn(e.getLocalizedMessage());
			}

			if (! ErrorCode.SUCCESS.equals(errorCode)) {
				future.completeExceptionally(VcxException.fromSdkError(err));

				return false;
			}

			return true;
		}

		//TODO: Is this redundant?
		/**
		 * Throws an VcxException if the provided error code does not indicate success.
		 * 
		 * @param err The error code to check.
		 * @throws VcxException Thrown if the error code does not indicate success.
		 */
		protected static void checkCallback(int err) throws VcxException {

			ErrorCode errorCode = ErrorCode.valueOf(err);
			if (! ErrorCode.SUCCESS.equals(errorCode)) throw VcxException.fromSdkError(err);
		}

		/**
		 * Throws an VcxException if the provided error code does not indicate success.
		 * 
		 * @param err The error code to check.
		 * @throws VcxException Thrown if the error code does not indicate success.
		 */
		protected static void checkResult(int err) throws VcxException {
			ErrorCode errorCode = ErrorCode.valueOf(err);
			if (! ErrorCode.SUCCESS.equals(errorCode)){
				throw VcxException.fromSdkError(err);
			} else{
				logger.debug("checkResult() returned: " + err);
			}
		}


		/**
		 * Sets the provided future with an exception if the error code provided does not indicate success.
		 *
		 * @param future The future.
		 * @param err    The error value to check.
		 * @return true if the error code indicated Success, otherwise false.
		 */
		protected static boolean checkResult(CompletableFuture<?> future, int err) {

			ErrorCode errorCode = ErrorCode.valueOf(err);
			if (! ErrorCode.SUCCESS.equals(errorCode)) {

				VcxException vcxException = VcxException.fromSdkError(err);
				future.completeExceptionally(vcxException);

				return false;
			}

			return true;
		}

		/*
		 * OBJECT METHODS
		 */

		@Override
		public int hashCode() {

			return HashCodeBuilder.reflectionHashCode(this, false);
		}

		@Override
		public boolean equals(Object other) {

			return EqualsBuilder.reflectionEquals(this, other, false);
		}

		@Override
		public String toString() {

			return ToStringBuilder.reflectionToString(this, ToStringStyle.SHORT_PREFIX_STYLE);
		}
	}

	/*
	 * JSON PARAMETER
	 */

	/**
	 * Base class for parameter objects that return JSON.
	 */
	public abstract static class JsonParameter {

		Map<String, Object> map = new HashMap<String, Object>();

		/*
		 * JSON CREATION
		 */

		/**
		 * Converts the map of parameters to a JSON string.
		 * 
		 * @return The JSON string.
		 */
		final String toJson() {

			StringBuilder builder = new StringBuilder();
			builder.append("{");

			for (Iterator<Map.Entry<String, Object>> iterator = this.map.entrySet().iterator(); iterator.hasNext(); ) {

				Map.Entry<String, Object> entry = iterator.next();
				String key = entry.getKey();
				Object value = entry.getValue();
				builder.append("\"").append(key).append("\":");
				if (value instanceof String) builder.append("\"").append(escapeJson(value.toString())).append("\"");
				else if (value instanceof Boolean) builder.append(value.toString());
				else if (value instanceof Number) builder.append(value.toString());
				else if (value == null) builder.append("null");
				else throw new IllegalArgumentException("Invalid value type: " + value + " (" + value.getClass() + ")");
				if (iterator.hasNext()) builder.append(",");
			}

			builder.append("}");

			return builder.toString();
		}

		private static String escapeJson(String string) {

			return string.replace("\\", "\\\\").replace("\"", "\\\"");
		}

		/*
		 * OBJECT METHODS
		 */

		@Override
		public int hashCode() {

			return this.map.hashCode();
		}

		@Override
		public boolean equals(Object other) {

			return this.map.equals(other);
		}

		@Override
		public String toString() {

			return this.toJson();
		}
	}

	/*
	 * Result
	 */

	public abstract static class Result {

		@Override
		public int hashCode() {

			return HashCodeBuilder.reflectionHashCode(this, false);
		}

		@Override
		public boolean equals(Object other) {

			return EqualsBuilder.reflectionEquals(this, other, false);
		}

		@Override
		public String toString() {

			return ToStringBuilder.reflectionToString(this, ToStringStyle.SHORT_PREFIX_STYLE);
		}
	}
}
