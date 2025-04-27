#!/bin/sh

# Variables

ARCH=$1
LIB_VER=$2
LIB=${3:-pandoc}
S3_BUCKET=${4:-cmp-lambda-deploy-cache}
ZIP_DIR=${5:-pandoc_cache}
ZIP_FILE=${ARCH}_${LIB}_${LIB_VER}_layer.zip
ZIP_FILE_PATH=${ZIP_DIR}/${ZIP_FILE}
LAYER_NAME=$(echo $ZIP_FILE | sed -e 's/_layer.zip//' -e 's/\./-/g')
S3_KEY="lambda-layers/$ZIP_FILE"

if [ ! -f "$ZIP_FILE_PATH" ]; then
  echo "$ZIP_FILE_PATH not found, check args!"
  exit 1
fi

# Sync the ZIP file to S3 (uploads only if different)
echo "Syncing layer ZIP $ZIP_FILE_PATH to S3..."
aws s3 sync $ZIP_DIR "s3://$S3_BUCKET/lambda-layers" \
  --exact-timestamps \
  --exclude "*" \
  --include "$ZIP_FILE"

# Check if the layer exists
EXISTING_LAYER=$(aws lambda list-layer-versions \
  --layer-name "$LAYER_NAME" \
  --query 'LayerVersions[0]' --output json 2>/dev/null)

#echo $EXISTING_LAYER

if [ "$EXISTING_LAYER" != "null" ]; then
  EXISTING_LAYER_VERSION=$(echo $EXISTING_LAYER | jq -r '.Version')
  EXISTING_LAYER_DATE=$(echo $EXISTING_LAYER | jq -r '.CreatedDate')
  echo "Layer $LAYER_NAME already exists with ver $EXISTING_LAYER_VERSION. Date: $EXISTING_LAYER_DATE"
  exit 1
else
  echo "Layer $LAYER_NAME does not exist. Creating new layer..."
fi

# Publish the Lambda layer from S3
PUBLISH_OUTPUT=$(aws lambda publish-layer-version \
  --layer-name "$LAYER_NAME" \
  --description "${LIB} lib/util, ver ${LIB_VER} for ${ARCH}" \
  --content S3Bucket="$S3_BUCKET",S3Key="$S3_KEY" \
  --query '{LayerArn:LayerArn, Version:Version}' --output json)

# Extract ARN and Version
LAYER_ARN=$(echo "$PUBLISH_OUTPUT" | jq -r '.LayerArn')
LAYER_VERSION=$(echo "$PUBLISH_OUTPUT" | jq -r '.Version')

# Check if layer was published successfully
if [ -z "$LAYER_ARN" ] || [ -z "$LAYER_VERSION" ]; then
  echo "Failed to publish layer"
  exit 1
fi

echo "Successfully published layer: $LAYER_NAME"
echo "Layer ARN: $LAYER_ARN"
echo "Layer Version: $LAYER_VERSION"
