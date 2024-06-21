//变量声明
const max_per_page = 20;
var page = 1;
var max_page = 2;
let JSONDATA;

function fetchData() {	
	fetch('http://127.0.0.1:12345/package_info/read')
		.then(response => {
			if (!response.ok) {
				throw new Error('Network response was not ok');
			}
			return response.json();
		})
		.then(data => {
			JSONDATA = data;
			
			max_page = Math.floor(data.length/max_per_page);
			if (data.length%max_per_page != 0) {
				max_page++;
			}
			
			showPage(data);
		})
		.catch(error => console.error('Error fetching data:', error));
}

function showPage(data) {
	//清空原数据
	document.getElementById("src_ip").innerText='';
	document.getElementById("src_port").innerText='';
	document.getElementById("dst_port").innerText='';
	document.getElementById("proto_type").innerText='';
	//显示页码
	document.getElementById("pages").innerHTML='<h3 style="margin-bottom:1;">' + page + '/' + max_page + '</h3>';
	//打印一页
	for(var i=page*max_per_page-max_per_page; i<page*max_per_page; i++){
		var existingElement = document.getElementById("src_ip");
		var appendedText = document.createTextNode(data[i].source_ip);
		existingElement.appendChild(appendedText);
		existingElement.appendChild(document.createElement("br"));

		existingElement = document.getElementById("src_port");
		appendedText = document.createTextNode(data[i].source_port);
		existingElement.appendChild(appendedText);
		existingElement.appendChild(document.createElement("br"));

		existingElement = document.getElementById("dst_port");
		appendedText = document.createTextNode(data[i].destination_port);
		existingElement.appendChild(appendedText);
		existingElement.appendChild(document.createElement("br"));

		existingElement = document.getElementById("proto_type");
		appendedText = document.createTextNode(data[i].proto_type);
		existingElement.appendChild(appendedText);
		existingElement.appendChild(document.createElement("br"));
	}
}

function pageUp() {
	if (page == 1) {
		alert("已经是第一页了");
	}else {
		page--;
		showPage(JSONDATA);
	}
}

function pageDown() {
	if (page == max_page) {
		alert("再往后就没有了");
	}else {
		page++;
		showPage(JSONDATA);
	}
}

document.addEventListener('DOMContentLoaded', fetchData);
setInterval(fetchData, 10000);