package utils;

public class State {
    public static class StateType {
        public static final int None = 0;
        public static final int Initialized = 1;
        public static final int OfferSent = 2;
        public static final int RequestReceived = 3;
        public static final int Accepted = 4;
        public static final int Unfulfilled = 5;
        public static final int Expired = 6;
        public static final int Revoked = 7;
        public static final int Redirected = 8;
        public static final int Rejected = 9;
    }

    public static class ProofState {
        public static final int Undefined = 0;
        public static final int Verified = 1;
        public static final int Invalid = 2;
    }
}
