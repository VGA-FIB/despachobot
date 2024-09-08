// Constants
const byte PROTOCOL_VERSION = 0; 

// Pins
const byte PIR_PIN = 4;

// Commands
const String PING_CMD = "ping";
const String GET_PROTOCOL_VERSION_CMD = "get_protocol_version";
const String GET_STATE_CMD = "get_state";
const String GET_LAST_FALLING_CMD = "get_last_falling";

// Globals
unsigned long last_pir_falling = 0;

void setup() {
  Serial.begin(115200);
  pinMode(PIR_PIN, INPUT);
  attachInterrupt(digitalPinToInterrupt(PIR_PIN), on_pir_falling, FALLING);
}

void loop() {
  String command = Serial.readStringUntil('\n');
  if (command == NULL) return;

  command.trim();
  if (command.length() == 0) return;

  if (command.equals(PING_CMD)) {
    Serial.println(1);
  } else if (command.equals(GET_PROTOCOL_VERSION_CMD)) {
    Serial.println(PROTOCOL_VERSION);
  } else if (command.equals(GET_STATE_CMD)) {
    Serial.println(digitalRead(PIR_PIN));
  } else if (command.equals(GET_LAST_FALLING_CMD)) {
    Serial.println(millis() - last_pir_falling);
  }
}

void on_pir_falling() {
  last_pir_falling = millis();
}
