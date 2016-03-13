#include "IrisComponents/IrisExpressions/IrisIdentifierExpression.h"
#include "IrisUnil/IrisIdentifier.h"
#include "IrisInstructorMaker.h"
#include "IrisCompiler.h"

const string & IrisIdentifierExpression::GetIdentifierString()
{
	return m_pIdentifier->GetIdentifierString();
}

bool IrisIdentifierExpression::Generate()
{
	IrisInstructorMaker* pMaker = IrisInstructorMaker::CurrentInstructor();
	IrisCompiler* pCompiler = IrisCompiler::CurrentCompiler();
	pCompiler->SetLineNumber(m_nLineNumber);

	unsigned int nIndex = pCompiler->GetIdentifierIndex(m_pIdentifier->GetIdentifierString(), pCompiler->GetCurrentFileIndex());

	switch (m_pIdentifier->GetType()) {
	case IrisIdentifilerType::Constance:
		pMaker->load(IrisAMType::Constance, nIndex);
		break;
	case IrisIdentifilerType::ClassVariable:
		pMaker->load(IrisAMType::ClassValue, nIndex);
		break;
	case IrisIdentifilerType::GlobalVariable:
		pMaker->load(IrisAMType::GlobalValue, nIndex);
		break;
	case IrisIdentifilerType::InstanceVariable:
		pMaker->load(IrisAMType::InstanceValue, nIndex);
		break;
	case IrisIdentifilerType::LocalVariable:
		pMaker->load(IrisAMType::LocalValue, nIndex);
		break;
	}

	return true;
}

bool IrisIdentifierExpression::LeftValue(IrisAMType & eType, IR_DWORD & bIndex)
{
	IrisInstructorMaker* pMaker = IrisInstructorMaker::CurrentInstructor();
	IrisCompiler* pCompiler = IrisCompiler::CurrentCompiler();
	pCompiler->SetLineNumber(m_nLineNumber);

	bIndex = pCompiler->GetIdentifierIndex(m_pIdentifier->GetIdentifierString(), pCompiler->GetCurrentFileIndex());

	switch (m_pIdentifier->GetType()) {
	case IrisIdentifilerType::Constance:
		eType = IrisAMType::Constance;
		break;
	case IrisIdentifilerType::ClassVariable:
		eType = IrisAMType::ClassValue;
		break;
	case IrisIdentifilerType::GlobalVariable:
		eType = IrisAMType::GlobalValue;
		break;
	case IrisIdentifilerType::InstanceVariable:
		eType = IrisAMType::InstanceValue;
		break;
	case IrisIdentifilerType::LocalVariable:
		eType = IrisAMType::LocalValue;
		break;
	}
	return true;
}

IrisIdentifierExpression::IrisIdentifierExpression(IrisIdentifier* pIdentifier) : m_pIdentifier(pIdentifier)
{
}


IrisIdentifierExpression::~IrisIdentifierExpression()
{
	if (m_pIdentifier)
		delete m_pIdentifier;

}
