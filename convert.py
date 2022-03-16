import tensorflow as tf
import tf2onnx

import train

model = tf.keras.models.load_model("target/model.h5")

model_proto, ext = tf2onnx.convert.from_keras(
    model,
    # input_signature = (train.INPUT_SIZE,),
    output_path = "target/model.onnx"
)
