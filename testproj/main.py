from time import sleep

print("YEAAAAA")
sleep(1)
print("WOOOOOOOO")

with open("yeet.txt", "w") as f:
    for i in range(1000):
        f.write("A"*1000)
        print("wrote," i)

exit(33)
