#ifndef _H_IRISSETTERSTATEMENT_
#define _H_IRISSETTERSTATEMENT_

#include "IrisStatement.h"

class IrisIdentifier;
class IrisBlock;
class IrisSetterStatement :
	public IrisStatement
{
protected:
	IrisIdentifier* m_pSetteredVariable = nullptr;
	IrisIdentifier* m_pParamName = nullptr;
	IrisBlock* m_pBlock = nullptr;

public:

	virtual bool Generate();

	IrisSetterStatement(IrisIdentifier* pSetteredVariable, IrisIdentifier* pParamName, IrisBlock* pBlock);
	virtual ~IrisSetterStatement();

	virtual bool Validate() override;
};

#endif