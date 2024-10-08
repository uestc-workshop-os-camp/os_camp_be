-- Your SQL goes here
CREATE TABLE if not exists phase2_user_info  (
  `id` int(10) UNSIGNED ZEROFILL NOT NULL,
  `username` varchar(255) CHARACTER SET utf8mb4 COLLATE utf8mb4_0900_ai_ci NOT NULL,
  `header_url` varchar(255) CHARACTER SET utf8mb4 COLLATE utf8mb4_0900_ai_ci NOT NULL,
  `ch3` double NOT NULL DEFAULT 0.0,
  `ch4` double NOT NULL DEFAULT 0.0,
  `ch5` double NOT NULL DEFAULT 0.0,
  `ch6` double NOT NULL DEFAULT 0.0,
  `ch8` double NOT NULL DEFAULT 0.0,
  `total` double NOT NULL DEFAULT 0.0,
  `pass_time` BIGINT NOT NULL,
  PRIMARY KEY (`id`,`username`) USING BTREE
) ENGINE = InnoDB AUTO_INCREMENT = 1 CHARACTER SET = utf8mb4 COLLATE = utf8mb4_0900_ai_ci ROW_FORMAT = Dynamic;

CREATE TABLE if not exists phase1_user_info  (
  `id` int(10) UNSIGNED ZEROFILL NOT NULL,
  `username` varchar(255) CHARACTER SET utf8mb4 COLLATE utf8mb4_0900_ai_ci NOT NULL,
  `header_url` varchar(255) CHARACTER SET utf8mb4 COLLATE utf8mb4_0900_ai_ci NOT NULL,
  `points` double NOT NULL DEFAULT 0.0,
  `total` double NOT NULL DEFAULT 0.0,
  `pass_time` BIGINT NOT NULL,
  PRIMARY KEY (`id`,`username`) USING BTREE
) ENGINE = InnoDB AUTO_INCREMENT = 1 CHARACTER SET = utf8mb4 COLLATE = utf8mb4_0900_ai_ci ROW_FORMAT = Dynamic;