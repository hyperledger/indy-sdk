#################################################################################################################################################
# post-archive-script.sh
#
# The purpose of this script is to create a universal binary for your framework
# Also - if there is a problem with steps in this script, then it should be
# easy to debug!  Other scripts that this is based off of aren't so easy to debug.
#
#################################################################################################################################################

# Validate that everything is setup correctly
validate() {
	# 1. Make sure the archival is coming from a workspace (not a project).
	# This is because a project doesn't provide enough enviornment variables to
	# correctly archive the project in the way we need to (yes, lazy programmer).
	if [ "${PROJECT_NAME}" == "" ]
	then
		exec > /tmp/${SCHEME_NAME}_archive.log 2>&1;
		echo "[ERROR]: PROJECT_NAME was not defined, did you select the 'Provide build settings from' in the 'Post-actions' for Archival?";
		envExit 1;
	else
		exec > ${TMPDIR}/${IBSC_MODULE}_archive.log 2>&1;
	fi

	# Make sure the project is setup to actually archive, otherwise we don't do anything.
	if [ "${ARCHIVE_PRODUCTS_PATH}" == "" ] ; then
		echo "[ERROR]: ARCHIVE_PRODUCTS_PATH is not defined - you probably need to set the SKIP_INSTALL build setting to NO"
		envExit 1;
	else
		echo "ARCHIVE_PRODUCTS_PATH=${ARCHIVE_PRODUCTS_PATH}"
	fi
}

# Configure some environment variables and ensure the universal folder exists
configure() {
	UNIVERSAL_OUTPUTFOLDER=${BUILD_DIR}/${CONFIGURATION}-universal
	if [ ! -d "${UNIVERSAL_OUTPUTFOLDER}" ]
	then
		echo "Created a UNIVERSAL_OUTPUTHOLDER: ${UNIVERSAL_OUTPUTFOLDER}"
		mkdir -p "${UNIVERSAL_OUTPUTFOLDER}"
	fi
	PROJECT_FOLDER=$(dirname "${PROJECT_FILE_PATH}");

	if [ "${CONFIGURATION}" == "" ]
	then
		echo "CONFIGURATION was not defined, setting it to Release"
		CONFIGURATION=Release
	fi
}
# If we're going to non-zero exit, lets print out the environment first (again,
# for debugging purposes, this is very useful).
envExit() {
	#env;
	exit $@;
}

# The work is done here, if we haven't been invoked yet, then go through the
# build steps, and dump the output to the log file.  You'll note that there are
# no comments in this function.  That's because the echo statements serve as
# those comments and provide breadcrumbs in the case that we can't build
buildFramework() {

	if [ "true" == ${ALREADYINVOKED:-false} ]
	then
		echo "RECURSION: Detected, stopping"
	else
		export ALREADYINVOKED="true"

		echo "Step 1. Building for iPhoneSimulator"
		xcodebuild -workspace "${WORKSPACE_PATH}" -scheme "${SCHEME_NAME}" -configuration ${CONFIGURATION} -sdk iphonesimulator -destination 'platform=iOS Simulator,name=iPhone 11' ONLY_ACTIVE_ARCH=NO ARCHS='x86_64' BUILD_DIR="${BUILD_DIR}" BUILD_ROOT="${BUILD_ROOT}" ENABLE_BITCODE=YES OTHER_CFLAGS="-fembed-bitcode" BITCODE_GENERATION_MODE=bitcode clean build
		if [ "$?" != "0" ]
		then
			echo "[ERROR]: FAILED Step 1: Building for iPhoneSimulator";
			envExit 1
		fi

		echo "Step 2. Copy the framework structure (from iphoneos build) to the universal folder"
		cp -R "${ARCHIVE_PRODUCTS_PATH}${INSTALL_PATH}" "${UNIVERSAL_OUTPUTFOLDER}/"
		if [ "$?" != "0" ]
		then
			echo "[ERROR]: FAILED Step 2: Copy the framework structure (from iphoneos build) to the universal folder.";
			envExit 1
		fi

		echo "Step 3. Copy Swift modules from iphonesimulator build (if it exists) to the copied framework directory"
		SIMULATOR_SWIFT_MODULES_DIR="${BUILD_DIR}/${CONFIGURATION}-iphonesimulator/${PRODUCT_NAME}.framework/Modules/${PRODUCT_NAME}.swiftmodule/."
		if [ -d "${SIMULATOR_SWIFT_MODULES_DIR}" ]; then
			cp -R "${SIMULATOR_SWIFT_MODULES_DIR}" "${UNIVERSAL_OUTPUTFOLDER}/${PRODUCT_NAME}.framework/Modules/${PRODUCT_NAME}.swiftmodule"
			if [ "$?" != "0" ]
			then
			echo "[ERROR]: FAILED Step 3: Copy Swift modules from iphonesimulator build (if it exists) to the copied framework directory";
			envExit 1
			fi
		fi

		echo "Step 4. Create universal binary file using lipo and place the combined executable in the copied framework directory"
		lipo -create -output "${UNIVERSAL_OUTPUTFOLDER}/${EXECUTABLE_PATH}" "${BUILD_DIR}/${CONFIGURATION}-iphonesimulator/${EXECUTABLE_PATH}" "${ARCHIVE_PRODUCTS_PATH}${INSTALL_PATH}/${EXECUTABLE_PATH}"
		if [ "$?" != "0" ]
		then
			echo "[ERROR]: FAILED Step 4: Create universal binary file using lipo and place the combined executable in the copied framework directory";
			envExit 1
		fi

		echo "Step 5. Create universal binaries for embedded frameworks";
		FRAMEWORKS_DIR="${UNIVERSAL_OUTPUTFOLDER}/${PRODUCT_NAME}.framework/Frameworks";
		if [ -e "${FRAMEWORKS_DIR}" ]
		then
			for SUB_FRAMEWORK in $(ls "${FRAMEWORKS_DIR}")
			do
				BINARY_NAME="${SUB_FRAMEWORK%.*}"
				lipo -create -output "${UNIVERSAL_OUTPUTFOLDER}/${PRODUCT_NAME}.framework/Frameworks/${SUB_FRAMEWORK}/${BINARY_NAME}" "${BUILD_DIR}/${CONFIGURATION}-iphonesimulator/${SUB_FRAMEWORK}/${BINARY_NAME}" "${ARCHIVE_PRODUCTS_PATH}${INSTALL_PATH}/${PRODUCT_NAME}.framework/Frameworks/${SUB_FRAMEWORK}/${BINARY_NAME}"
				if [ "$?" != "0" ]
				then
					echo "[ERROR]: FAILED Step 5 (${SUB_FRAMEWORK}): Create universal binaries for embedded frameworks";
					envExit 1
				fi
			done
		fi

		echo "Step 6. Convenience step to copy the framework to the project's directory"
		yes | cp -Rf "${UNIVERSAL_OUTPUTFOLDER}/${FULL_PRODUCT_NAME}" "${ARCHIVE_PRODUCTS_PATH}${INSTALL_PATH}" && \
		yes | cp -Rf "${UNIVERSAL_OUTPUTFOLDER}/${FULL_PRODUCT_NAME}" "${PROJECT_FOLDER}"
		if [ "$?" != "0" ]
		then
			echo "[ERROR]: FAILED Step 6: Convenience step to copy the framework to the project's directory";
			envExit 1;
		fi

	fi
}

archiveFramework() {

	echo "Step 7. Create a zip archive of the framework"
	if [ ! -e "${PROJECT_FOLDER}/${IBSC_MODULE}.framework" ]
	then
		echo "[ERROR]: FAILED Step 7: ${IBSC_MODULE}/${PRODUCT_NAME}.framework doesn't exist";
		envExit 1;
	fi

	cd "${PROJECT_FOLDER}"
	#VERSION=$(grep s.version ../${PRODUCT_NAME}.podspec|grep -v s.source|grep -v cocoapods_version|sed 's/s.version//'|sed 's/[^\.,0-9]//g');
	zip -r "${IBSC_MODULE}-${TARGET_NAME}.framework.zip" "${IBSC_MODULE}.framework"
}

# main function - delegates to the helpers
main() {
	validate;
	configure;
	buildFramework;
	archiveFramework;

	open "${PROJECT_FOLDER}";
}

main;
