# 運行時 (Actor VM)
Actor VM將會是一個特別的運行時，一個雲原生的運行時，它將原生支持FaaS。它會是一個厚重的運行時，可以直接作為容器被cgroups和k8s運行。甚至能作為虛擬機直接啓動於QEMU中，並直接在運行時上實現container cri，可直接被k8s等技術管理。

# 語言特性
1. 函数式編程
2. 模式匹配
3. 鏈式調用
4. Actor 並發模型
5. 靜態強類型
6. 柯里化

# Actor VM
Actor VM 是雲原生面向多租戶的虛擬機，因此沒有本地IO，一切都透過Actor和message來進行溝通。廠商應該提供與Actor溝通的方法和實現。
該虛擬機實現了兩個特殊的指令，`send`用於發送message`receive`用於接收message`spawn`用於創建新的Actor。
Actor Vm是一個寄存器型的虛擬機，有11個寄存器。
## 基本數據類型
| 類型 | 指令 | 說明 |
| --- | --- | --- |
| Int | INT | |
| Float | FLO | |
| String | STR | |
| Atom | ATM ||
| Tuple | TUP ||
| List | LIST ||
| Map | MAP ||
## 寄存器
| 寄存器 | 說明 |
| ---- | ---- |
| R0 | 通用寄存器 |
| R1 | 通用寄存器 |
| R2 | 通用寄存器 |
| R3 | 通用寄存器 |
| R4 | 通用寄存器 |
| R5 | 通用寄存器 |
| R6 | 通用寄存器 |
| R7 | 通用寄存器 |
| RM | 用於保存接收到的信息 |
| PC | 下一個指令的地址 |
| ZF | 上一次邏輯運算的結果 |
| LR | 棧的返回地址 |
## Assembly
```
MOVE R0, R1 ; 移動R0到R1
STORE R0, 0x1234 ;
LOAD R0, 0x1234 ;
ADD R0, R1, R2 ; R1加R2存到R0
SUB R0, R1, R2 ; R1減R2存到R0
MUL R0, R1, R2
DIV R0, R1, R2
MOD R0, R1, R2
EQ R0, R1 ; 如果R0等於R1就將1存到ZF不等於就將0存到ZF
NE R0, R1 ; 如果R0等於R1就將0存到ZF不等於就將1存到ZF
GT R0, R1 ;
LT R0, R1 ;
GTE R0, R1 ;
LTE R0, R1 ;
label: ; 標記記憶體位置用於跳轉
JUMP <label> ; 跳到label
JUMPIF <label> ; 如果ZF是1就跳到label
PUSH R0 ; 將R0推入棧中
POP R0 ; 彈出棧中的值到R0
INT R0, <int64> ; 將int64整數放入R0
STR R0, <string> ; 將字串放入R0中
FLO R0, <float64> ; 將float64浮點數放入R0
ATM R0, <string> ; 將字串作為Atom放入R0
TUP R0, <int64> ; 將size tuple放入R0
LIST R0, <int64> ; 創建一個長度為<int64>的list放入R0
SIZE R0, R1 ; 計算R0的長度放到R1，可用於tuple, list, string, map
MAP R0 ; 創建一個map放入R0
SET_C R0, R1, R2 ; 將R0的R1索引設置為R2，可用於tuple, list string, map
MOV_C R0, R1, R2 ; 將R0的R1索引中的值移到R2，可用於tuple, list string, map
SEND R0, R1 ; 將R0將R0發送至R1所指的地址，由供應商提供。
RECV ; 等待信息
HLT ; 
```

## Message
發送和接收message應該由供應商提供，當調用SEND

## Message Assembly
發送的信息可以以二進制的Assembly形式傳送，以下是在傳送時使用的指令，虛擬機提供一块安全的記憶體供操作，最終將message存入RM寄存器。
```
MOVE R0, R1 ;
STORE R0, 0x1234 ;
LOAD R0, 0x1234  ;
INT R0, <int64> ; 將int64整數放入R0
STR R0, <字串> ; 將長度為len的字串放入R0中
FLO R0, <float64> ; 將float64浮點數放入R0
```
```
clang -S -target arm-linux-gnueabihf -march=armv7-a main.c -o example_arm.s
```