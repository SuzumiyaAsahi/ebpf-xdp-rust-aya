//IPV4合法性校验
function isValidIPv4(ip) {
	var pattern = /^(25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.(25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.(25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.(25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)$/;
	return pattern.test(ip);
}
//解禁选中
function unbanSelected() {
	// 构造列表
	var tar = [];
	// 获取所有复选框元素
	var checkboxes = document.querySelectorAll("#ban_list input[type=checkbox]");

	// 遍历复选框元素
	checkboxes.forEach(function(checkbox) {
		// 如果复选框被选中
		if (checkbox.checked) {
			//alert(checkbox.nextSibling.nodeValue);
			var jsonItem = {'ipv4':checkbox.nextSibling.nodeValue};
			tar.push(jsonItem);
		}
	});
	unbanIP(tar);
	//alert(JSON.stringify(tar));// for test
	tar = [];
}
//从输入框加入Temporary
function addBan() {
	// 获取输入框中的字串
	var inputString = document.getElementById("banName").value;

	// 验证 IPv4 地址格式
	if (!isValidIPv4(inputString)) {
		document.getElementById("banName").value = "";
		alert("请输入有效的 IPv4 地址！");
		return;
	}
	// 验证是否已在temp列表
	var paragraphs = document.getElementById("temp").getElementsByTagName("p");
	let had = false;
	for (var i = 0; i < paragraphs.length; i++) {
        // 获取段落元素的文本内容
		if(inputString === paragraphs[i].textContent) {
			alert('重复IP');
			had = true;
			break;
		}
    }
	if(!had) {
		// 创建一个新的段落元素
		var newParagraph = document.createElement("p");

		// 将输入的字串添加到段落元素中
		newParagraph.textContent = inputString;

		// 创建一个复选框元素
		var checkbox = document.createElement("input");
		checkbox.type = "checkbox";

		// 将复选框添加到段落元素前面
		newParagraph.insertBefore(checkbox, newParagraph.firstChild);

		// 获取输出区域
		var outputDiv = document.getElementById("temp");

		// 将新的段落元素添加到输出区域中
		outputDiv.appendChild(newParagraph);
	}

	// 清空输入框
	document.getElementById("banName").value = "";
}
//从Temporary删除
function deleteSelected() {
	// 获取所有复选框元素
	var checkboxes = document.querySelectorAll("#temp input[type=checkbox]");

	// 遍历复选框元素
	checkboxes.forEach(function(checkbox) {
		// 如果复选框被选中
		if (checkbox.checked) {
			// 删除该复选框所在的段落元素
			checkbox.parentNode.remove();
		}
	});
}
//提交Temporary
function submitBan() {
	// 构造列表
	var tar = [];
	// 获取输出区域
	var outputDiv = document.getElementById("temp");
	// 获取输出区域内所有段落元素
    var paragraphs = outputDiv.getElementsByTagName("p");

	// 遍历所有段落元素
    for (var i = 0; i < paragraphs.length; i++) {
        // 获取段落元素的文本内容
        var paragraphText = paragraphs[i].textContent;
		var jsonItem = {'ipv4':paragraphText};
		if (BANLIST.length != 0) {
			//去除已禁用的IP
			var alreadyExist = BANLIST.some(function(item) {
				return item.ipv4 === paragraphText;
			})
		}
		//应该还要过滤一下F12的绕过,但是后端过滤了，这算攻击了吧
		if (!alreadyExist /*&& isValidIPv4(paragraphText)*/) {
			tar.push(jsonItem);
		}
    }
	if (tar.length != 0) {
		alert("防火墙自动重启后生效");
		banIP(tar);
	}
	//清空列表
	tar = [];
	document.getElementById("temp").innerText='';
}

function banIP(aim) {
	fetch('http://127.0.0.1:12345/blocked_ip/write_many', {
		method: 'POST',
		headers: {
			'Content-Type': 'application/json'
		},
		body: JSON.stringify(aim)
	})
	.then(response => {
		if (!response.ok) {
			throw new Error('出错');
		}
		//重启后刷新页面
		setTimeout(location.reload(),2000);
		return response.json();
	})
}

function unbanIP(aim) {
	fetch('http://127.0.0.1:12345/blocked_ip/delete_many', {
		method: 'DELETE',
		headers: {
			'Content-Type': 'application/json'
		},
		body: JSON.stringify(aim)
	})
	.then(response => {
		if (!response.ok) {
			throw new Error('出错');
		}
		//重启后刷新页面
		setTimeout(location.reload(),2000);
		return response.json();
	})
}

function unbanALL() {
	fetch('http://127.0.0.1:12345/blocked_ip/flush', {
		method: 'DELETE',
	})
	.then(response => {
		if (!response.ok) {
			throw new Error('出错');
		}
		//重启后刷新页面
		setTimeout(location.reload(),2000);
		return response.json();
	})
}

//**************************Print*************************************

const max_per_pageB = 20;
var pageB = 1;
var max_pageB = 2;
let BANLIST=[];

function getBannedList() {
	fetch('http://127.0.0.1:12345/blocked_ip/read')
		.then(response => {
			if (!response.ok) {
				throw new Error('Network response was not ok');
			}
			return response.json();
		})
		.then(data => {
			document.getElementById("banNum").innerHTML='<h3 style="margin:0;">共'+ data.length +'个<h3>';
			if (Object.keys(data).length === 0) {
			  // JSON文件为空的情况
			  document.getElementById("ban_list").innerText='';
			} else {
				BANLIST = data;
				
				document.getElementById("ban_list").innerText = "";
				for (var i = 0; i < data.length; i++) {
					// 创建一个新的段落元素
					var newParagraph = document.createElement('p');

					// 将输入的字串添加到段落元素中
					newParagraph.textContent = data[i].ipv4;

					// 创建一个复选框元素
					var checkbox = document.createElement("input");
					checkbox.type = "checkbox";

					// 将复选框添加到段落元素前面
					newParagraph.insertBefore(checkbox, newParagraph.firstChild);

					// 获取输出区域
					var outputDiv = document.getElementById("ban_list");

					// 将新的段落元素添加到输出区域中
					outputDiv.appendChild(newParagraph);
				}

			}
		})
		.catch(error => console.error('Error fetching data:', error));
}

function pageUpB() {
	if (pageB == 1) {
		alert("已经是第一页了");
	}else {
		pageB--;
		showpage(BANLIST);
	}
}

function pageDownB() {
	if (pageB == max_pageB) {
		alert("再往后就没有了");
	}else {
		pageB++;
		showpage(BANLIST);
	}
}


document.addEventListener('DOMContentLoaded', getBannedList);