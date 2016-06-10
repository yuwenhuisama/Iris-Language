#include "IrisUnil/IrisVirtualCodeFile.h"
#include <sys/stat.h>
using namespace std;


bool IrisVirtualCodeFile::_WriteFileBody(fstream & fOutFile, const IrisCompiler::_StatementInfo * pInfo)
{
	// File Header
	fOutFile.write((char*)&m_fhHeader, sizeof(_FileHeader));

	// String Field
	for (auto& strElem : *pInfo->m_pStringSpace) {
#ifdef IR_USE_STL_STRING
		_WriteString(fOutFile, strElem);
#else
		_WriteString(fOutFile, strElem.GetSTLString());
#endif // IR_USE_STL_STRING
	}

	// Unique String Field
	for (auto& strElem : *pInfo->m_pUniqueStringSpace) {
#ifdef IR_USE_STL_STRING
		_WriteString(fOutFile, strElem);
#else
		_WriteString(fOutFile, strElem.GetSTLString());
#endif // IR_USE_STL_STRING
	}

	// Integer Field
	for (auto& nInt : *pInfo->m_pIntegerSpace) {
		fOutFile.write((char*)&nInt, sizeof(int));
	}

	// Float Field
	for (auto& nFloat : *pInfo->m_pFloatSpace) {
		fOutFile.write((char*)&nFloat, sizeof(double));
	}

	// Identifier Field
	for (auto& nIden : *pInfo->m_pIdentifierSpace) {
#ifdef IR_USE_STL_STRING
		_WriteString(fOutFile, nIden);
#else
		_WriteString(fOutFile, nIden.GetSTLString());
#endif // IR_USE_STL_STRING
	}

	// Virtual Code
	for (auto& nCode : *pInfo->m_pCodes) {
		fOutFile.write((char*)&nCode, sizeof(decltype(nCode)));
	}

	return true;
}

bool IrisVirtualCodeFile::_WriteString(fstream & fOutFile, const string & strInput)
{
	auto nSize = strInput.size();
	fOutFile.write((char*)&nSize, sizeof(size_t));
	fOutFile.write(strInput.c_str(), nSize);
	return true;
}

bool IrisVirtualCodeFile::_ReadFileBody(fstream & fInFile, IrisCompiler::_StatementInfo * pInfo)
{
	// String Field
	for (int i = 0;i < m_fhHeader.m_nStringFieldElemCount; ++i) {
		string strTmp;
		_ReadString(fInFile, strTmp);
		pInfo->m_pStringField->insert(pair<string, int>(strTmp, i));
		pInfo->m_pStringSpace->push_back(strTmp);
	}

	// Unique String Field
	for (int i = 0;i < m_fhHeader.m_nUniqueFieldElemCount; ++i) {
		string strTmp;
		_ReadString(fInFile, strTmp);
		pInfo->m_pUniqueStringField->insert(pair<string, int>(strTmp, i));
		pInfo->m_pUniqueStringSpace->push_back(strTmp);
	}

	// Integer Field
	for (int i = 0;i < m_fhHeader.m_nIntegerFieldElemCount; ++i) {
		int nData = 0;
		fInFile.read((char*)&nData, sizeof(int));
		pInfo->m_pIntegerField->insert(pair<int, int>(nData, i));
		pInfo->m_pIntegerSpace->push_back(nData);
	}

	// Float Field
	for (int i = 0;i < m_fhHeader.m_nFloatFieldElemCount; ++i) {
		double dData = 0.0;
		fInFile.read((char*)&dData, sizeof(double));
		pInfo->m_pFloatField->insert(pair<double, int>(dData, i));
		pInfo->m_pFloatSpace->push_back(dData);
	}

	// Identifier Field
	for (int i = 0; i < m_fhHeader.m_nIdentifierFieldElemCount; ++i) {
		string strTmp;
		_ReadString(fInFile, strTmp);
		pInfo->m_pIdentifierField->insert(pair<string, int>(strTmp, i));
		pInfo->m_pIdentifierSpace->push_back(strTmp);
	}

	// Virtual Code
	for (int i = 0; i < m_fhHeader.m_nVirtualCodeCount; ++i) {
		IR_WORD wData = 0;
		fInFile.read((char*)&wData, sizeof(IR_WORD));
		pInfo->m_pCodes->push_back(wData);
	}

	return true;
}

bool IrisVirtualCodeFile::_ReadString(fstream & fInFile, string & strGet)
{
	size_t nSize = 0;
	fInFile.read((char*)&nSize, sizeof(size_t));
	char* pBuffer = new char[nSize];
	fInFile.read(pBuffer, nSize);
	strGet.assign(pBuffer, nSize);
	delete[] pBuffer;
	return true;
}

bool IrisVirtualCodeFile::SaveToFile(const string& strOrgFileName, const IrisCompiler::_StatementInfo* pInfo)
{
	m_fhHeader.m_nFloatFieldElemCount = pInfo->m_pFloatSpace->size();
	m_fhHeader.m_nIdentifierFieldElemCount = pInfo->m_pIdentifierSpace->size();
	m_fhHeader.m_nIntegerFieldElemCount = pInfo->m_pIntegerSpace->size();
	m_fhHeader.m_nStringFieldElemCount = pInfo->m_pStringSpace->size();
	m_fhHeader.m_nUniqueFieldElemCount = pInfo->m_pUniqueStringSpace->size();
	m_fhHeader.m_nVirtualCodeCount = pInfo->m_pCodes->size();

	fstream fOrgFile(strOrgFileName);
	fstream fDestFile;
	string strDestFileName;
	// Virtual Script do not create .irc file
	if (!fOrgFile) {
		return true;
	}
	// Real Script
	else {
		fOrgFile.close();
		struct stat sState;
		stat(strOrgFileName.c_str(), &sState);
		//m_fhHeader.m_nTimeStamp = sState.st_mtime;

		auto nPos = strOrgFileName.find_last_of(".");
		if (nPos != std::string::npos) {
			strDestFileName.assign(strOrgFileName, 0, nPos);
		}
		strDestFileName += ".irc";

		fstream fTryDestFile(strDestFileName, ios::in | ios::binary);
		if (fTryDestFile) {
			_FileHeader fhTmp;
			fTryDestFile.read((char*)&fhTmp, sizeof(_FileHeader));
			fTryDestFile.close();

			if (string(fhTmp.m_szMagicWord, 4) == string("ircf") && fhTmp.m_nVersion == 0x00000001) {
				//if not modified
				if (fhTmp.m_nTimeStamp == sState.st_mtime) {
					return true;
				}
			}
		}

		m_fhHeader.m_nTimeStamp = sState.st_mtime;
	}
	
	fDestFile.open(strDestFileName.c_str(), ios::binary | ios::out | ios::trunc);
	//fDestFile.clear();

	_WriteFileBody(fDestFile, pInfo);

	fDestFile.close();

	return true;
}

bool IrisVirtualCodeFile::LoadFromFile(const string& strOrgFileName, IrisCompiler::_StatementInfo* pInfo)
{
	fstream fOrgFile(strOrgFileName, ios::in | ios::binary);
	// not exist
	if (!fOrgFile) {
		return false;
	}
	fOrgFile.close();
	struct stat sState;
	stat(strOrgFileName.c_str(), &sState);
	auto nOrgTimeStamp = sState.st_mtime;

	string strDestFileName;
	auto nPos = strOrgFileName.find_last_of(".");
	if (nPos != std::string::npos) {
		strDestFileName.assign(strOrgFileName, 0, nPos);
	}
	strDestFileName += ".irc";

	fstream fDestFile(strDestFileName, ios::in | ios::binary);
	// File not exist
	if (!fDestFile) {
		return false;
	}
	_FileHeader fhTmp;
	fDestFile.read((char*)&fhTmp, sizeof(_FileHeader));

	// not modified
	if (fhTmp.m_nTimeStamp == nOrgTimeStamp) {
		m_fhHeader = fhTmp;
		pInfo->m_pCodes->reserve(fhTmp.m_nVirtualCodeCount);
		pInfo->m_pFloatSpace->reserve(fhTmp.m_nFloatFieldElemCount);
		pInfo->m_pIdentifierSpace->reserve(fhTmp.m_nIdentifierFieldElemCount);
		pInfo->m_pIntegerSpace->reserve(fhTmp.m_nIntegerFieldElemCount);
		pInfo->m_pStringSpace->reserve(fhTmp.m_nStringFieldElemCount);
		pInfo->m_pUniqueStringSpace->reserve(fhTmp.m_nUniqueFieldElemCount);
		_ReadFileBody(fDestFile, pInfo);
		fDestFile.close();

	}
	else {
		fDestFile.close();
		return false;
	}

	return true;
}

IrisVirtualCodeFile::IrisVirtualCodeFile()
{
}


IrisVirtualCodeFile::~IrisVirtualCodeFile()
{
}