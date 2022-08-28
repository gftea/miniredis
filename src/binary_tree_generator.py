import random
# script to generate tets vector for below leetcode problem
# https://leetcode.com/problems/maximum-depth-of-binary-tree/
#
#
def random_node() -> int:
    n = random.randint(-110,110)
    if n >= -100 and n <= 100:
        return n
    return "null"

def add_node(arr: list, nodes: list, limit: int):
    before = len(arr)
    if before >= limit: return

    remaining = limit - before
    for node in nodes:
        if node != "null":
            # left leaf node
            arr.append(random_node())
            # right leaf node
            arr.append(random_node())
            remaining -= 2
            if remaining <= 0: return

    after = len(arr)
    # print(before, after, arr[before:after])
    add_node(arr, arr[before:after], limit)

def generate_binary_tree_array(limit: int) -> list:
    # result array
    arr = list()

    # root node
    root = 0
    add_node(arr, [root], limit)
    return arr

if __name__ == "__main__":
    result = generate_binary_tree_array(10000)
    # remove quote of "null"
    # convert all elements to str, then join them as a single str
    print("[", ", ".join(map(str, result)), "]")