CREATE TABLE IF NOT EXISTS "apc_reading" (
    "uuid" UUID NOT NULL PRIMARY KEY DEFAULT gen_random_uuid(),
    "created_on" TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "measurement_time" TIMESTAMP WITH TIME ZONE NOT NULL,
    "location" TEXT NOT NULL,
    "device_sn" TEXT NOT NULL,
    "tvoc" INT NOT NULL,
    "eco2" INT NOT NULL,
    "aqi" INT NOT NULL,
    "temperature" INT NOT NULL,
    "humidity" INT NOT NULL,
    "pm1_0" INT NOT NULL,
    "pm2_5" INT NOT NULL,
    "pm10" INT NOT NULL,
    "pm1_0_in_air" INT NOT NULL,
    "pm2_5_in_air" INT NOT NULL,
    "pm10_in_air" INT NOT NULL,
    "um0_3_particles" INT NOT NULL,
    "um0_5_particles" INT NOT NULL,
    "um1_particles" INT NOT NULL,
    "um2_5_particles" INT NOT NULL,
    "um5_particles" INT NOT NULL,
    "um10_particles" INT NOT NULL
);

CREATE INDEX "measurement_time_dev_index" ON "apc_reading" ("measurement_time", "device_sn");
