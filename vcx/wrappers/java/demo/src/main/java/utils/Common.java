package utils;

import com.google.gson.Gson;
import com.google.gson.GsonBuilder;
import com.google.gson.JsonParser;
import com.sun.jna.Library;
import com.sun.jna.Native;
import org.apache.commons.cli.*;

import java.util.Random;
import java.util.logging.*;

public class Common {
    public static void setLibraryLogger(String logLevel) {
        System.setProperty(org.slf4j.impl.SimpleLogger.DEFAULT_LOG_LEVEL_KEY, logLevel);
    }

    public static class LogFormatter extends Formatter {
        public String format(LogRecord rec) {
            StringBuffer buf = new StringBuffer(1000);

            buf.append("[").append(rec.getSourceMethodName()).append("] ");
            buf.append(rec.getLevel()).append(" ").append(rec.getSourceClassName()).append(" - ");
            buf.append(rec.getMessage()).append("\n");

            return buf.toString();
        }
    }

    public static Logger getDemoLogger() {
        // remove rootLogger
        Logger rootLogger = Logger.getLogger("");
        Handler[] handlers = rootLogger.getHandlers();
        if (handlers[0] instanceof ConsoleHandler) {
            rootLogger.removeHandler(handlers[0]);
        }

        Logger logger = Logger.getGlobal();
        logger.setLevel(Level.INFO);

        Handler handler = new ConsoleHandler();
        handler.setFormatter(new LogFormatter());
        logger.addHandler(handler);

        return logger;
    }

    public static String prettyJson(String jsonString) {
        Gson gson = new GsonBuilder().setPrettyPrinting().create();
        return gson.toJson(JsonParser.parseString(jsonString));
    }

    private interface NullPayApi extends Library {
        public int nullpay_init();
    }

    public static void loadNullPayPlugin(){
        NullPayApi nullPayApi = Native.loadLibrary("nullpay", NullPayApi.class);
        nullPayApi.nullpay_init();
    }

    private interface PostgresApi extends Library {
        public int postgresstorage_init();
    }

    public static void loadPostgresPlugin(){
        PostgresApi postgresApi = Native.loadLibrary("indystrgpostgres", PostgresApi.class);
        postgresApi.postgresstorage_init();
    }

    public static int getRandomInt(int min, int max) {
        if (min >= max)
            throw new IllegalArgumentException("max must be greater than min");
        Random r = new Random();
        return r.nextInt((max - min) + 1) + min;
    }

    public static CommandLine getCommandLine(String[] args) {
        Option help = new Option("h", "help", false, "Display this usage guide.");
        Option postgres = new Option("p", "postgres", false,"If specified, postgres wallet will be used.");

        Options options = new Options();
        options.addOption(help);
        options.addOption(postgres);

        CommandLineParser parser = new DefaultParser();
        try {
            CommandLine line = parser.parse(options, args);
            if(line.hasOption("help")) {
                HelpFormatter formatter = new HelpFormatter();
                formatter.printHelp( "task", options );
                return null;
            }
            return line;
        } catch (ParseException exp) {
            System.err.println("Parsing failed. Reason: " + exp.getMessage());
        }
        return null;
    }
}
